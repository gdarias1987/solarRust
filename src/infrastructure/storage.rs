use crate::{
    Error, Result,
    domain::{Archive, Record, catalog},
};
use std::{
    fs::{self, File, OpenOptions},
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

pub struct Store {
    pub root: PathBuf,
    _lock: File,
}

pub fn data_directory() -> PathBuf {
    if let Some(path) = std::env::var_os("SOLAR_DATA_DIR") {
        return path.into();
    }
    if let Some(path) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(path).join("SolarObservatory");
    }
    if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(path).join("solar-observatory");
    }
    PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| ".".into()))
        .join(".local/share/solar-observatory")
}

impl Store {
    pub fn open(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join("instance.lock"))?;
        lock.try_lock().map_err(|e| {
            Error::Invalid(format!(
                "Data directory is already in use or cannot be locked: {e}"
            ))
        })?;
        Ok(Self { root, _lock: lock })
    }
    pub fn path(&self, file: &str) -> Result<PathBuf> {
        if file.is_empty() || file.contains(['/', '\\', ':']) || file.starts_with('.') {
            return Err(Error::Invalid("Unsafe local image reference".into()));
        }
        Ok(self.root.join(file))
    }
    pub fn load(&self) -> Result<Archive> {
        let path = self.root.join("archive.json");
        let mut archive: Archive = match fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Archive::default(),
            Err(e) => return Err(e.into()),
        };
        archive.preferences.validate()?;
        if let Some(c) = &archive.preferences.crop {
            self.path(&c.source)?;
        }
        let before = archive.records.len();
        for r in &archive.records {
            catalog::channel(&r.channel)?;
            self.path(&r.file)?;
            if r.width == 0 || r.height == 0 {
                return Err(Error::Invalid("Invalid image dimensions in archive".into()));
            }
        }
        archive.records.retain(|r| {
            let valid = self
                .path(&r.file)
                .ok()
                .and_then(|p| fs::metadata(p).ok())
                .is_some_and(|m| m.is_file() && m.len() == r.bytes);
            if !valid {
                eprintln!("Unavailable archive image: {}", r.file);
            }
            valid
        });
        // Persist the pruned manifest: otherwise every launch re-checks files that are gone.
        if archive.records.len() != before {
            self.save(&archive)?;
        }
        Ok(archive)
    }
    pub fn save(&self, archive: &Archive) -> Result<()> {
        archive.preferences.validate()?;
        self.atomic_write("archive.json", &serde_json::to_vec_pretty(archive)?)
    }
    pub fn atomic_write(&self, name: &str, bytes: &[u8]) -> Result<()> {
        let path = self.path(name)?;
        let mut file = tempfile::NamedTempFile::new_in(&self.root)?;
        file.write_all(bytes)?;
        file.as_file().sync_all()?;
        file.persist(path).map_err(|e| Error::Storage(e.error))?;
        Ok(())
    }
    pub fn add(&self, archive: &mut Archive, record: Record, bytes: &[u8]) -> Result<()> {
        // Commit the complete file before publishing its record. A failed manifest leaves
        // an unreferenced file, never a record pointing at a partial download.
        self.atomic_write(&record.file, bytes)?;
        let mut next = archive.clone();
        next.records.push(record);
        self.save(&next)?;
        *archive = next;
        Ok(())
    }
    pub fn derive(&self, record: &Record, crop: &crate::domain::crop::Crop) -> Result<PathBuf> {
        let original = decode(&fs::read(self.path(&record.file)?)?)?;
        let (x, y, w, h) = crop.scaled(original.width(), original.height())?;
        let output = original.crop_imm(x, y, w, h).to_rgb8();
        let mut encoded = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(output).write_to(&mut encoded, image::ImageFormat::Jpeg)?;
        let name = format!("crop-{}-{}.jpg", record.id, unique_id());
        self.atomic_write(&name, encoded.get_ref())?;
        self.path(&name)
    }
    /// Cropped derivatives are regenerable: they carry a fresh name on every apply so the
    /// desktop reloads them, which means the previous one is only waste. Keep `keep` (the file
    /// the desktop is showing) and delete the rest.
    pub fn sweep_derivatives(&self, keep: &[String]) {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if name.starts_with("crop-") && !keep.iter().any(|kept| kept == name) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

pub fn unique_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    format!(
        "{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}
pub fn decode(bytes: &[u8]) -> Result<image::DynamicImage> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    Ok(reader.decode()?)
}
pub fn preview(path: &Path) -> Result<(u32, u32, Vec<u8>)> {
    let image = decode(&fs::read(path)?)?.thumbnail(640, 640).to_rgba8();
    Ok((image.width(), image.height(), image.into_raw()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(dir: &tempfile::TempDir) -> Store {
        Store::open(dir.path().to_path_buf()).expect("a fresh directory opens")
    }

    #[test]
    fn load_persists_a_pruned_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(&dir);
        let mut archive = Archive::default();
        archive.records.push(Record {
            id: "gone".into(),
            channel: "0304".into(),
            url: String::new(),
            remote_modified: None,
            downloaded_at: 1,
            bytes: 4,
            file: "0304-gone.jpg".into(),
            width: 1,
            height: 1,
        });
        store.save(&archive).unwrap();
        assert!(store.load().unwrap().records.is_empty());
        let reread: Archive =
            serde_json::from_slice(&fs::read(dir.path().join("archive.json")).unwrap()).unwrap();
        assert!(
            reread.records.is_empty(),
            "the pruned manifest is written back"
        );
    }

    #[test]
    fn sweep_keeps_the_active_derivative_and_never_a_capture() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("crop-a.jpg"), b"a").unwrap();
        fs::write(dir.path().join("crop-b.jpg"), b"b").unwrap();
        fs::write(dir.path().join("0304-capture.jpg"), b"c").unwrap();
        store(&dir).sweep_derivatives(&["crop-b.jpg".into()]);
        assert!(!dir.path().join("crop-a.jpg").exists());
        assert!(dir.path().join("crop-b.jpg").exists());
        assert!(dir.path().join("0304-capture.jpg").exists());
    }
}

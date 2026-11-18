//! One worker owns the archive. UI commands and scheduled work are serialized.
use crate::{
    Error, Result,
    domain::{Archive, Language, Mode, Record, catalog, crop::Crop, lifecycle},
    infrastructure::{
        nasa::{Fetch, ImageSource},
        storage::{Store, decode, unique_id},
        wallpaper,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    NotRun,
    Succeeded,
    Unchanged,
    Failed(String),
}
#[derive(Clone, Debug)]
pub struct Outcome {
    pub channel: String,
    pub modified: Option<String>,
    pub check: Stage,
    pub download: Stage,
    pub persistence: Stage,
    pub apply: Stage,
}
impl Outcome {
    pub fn new(channel: &str) -> Self {
        Self {
            channel: channel.into(),
            modified: None,
            check: Stage::NotRun,
            download: Stage::NotRun,
            persistence: Stage::NotRun,
            apply: Stage::NotRun,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Preference {
    Channel(String),
    Language(Language),
    Mode(Mode),
    Crop(Option<Crop>),
    Interval(u64),
    Retention(usize),
}
pub struct Service<S> {
    pub store: Store,
    pub archive: Archive,
    pub source: S,
}
impl<S: ImageSource> Service<S> {
    pub fn open(store: Store, source: S) -> Result<Self> {
        let archive = store.load()?;
        // A crop derivative is regenerable, so only the one the desktop shows survives a
        // restart. Unknown active wallpaper means nothing is safe to remove.
        if let Some(active) = wallpaper::active()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        {
            store.sweep_derivatives(&[active]);
        }
        Ok(Self {
            store,
            archive,
            source,
        })
    }
    pub fn preference(&mut self, preference: Preference) -> Result<()> {
        let mut next = self.archive.clone();
        match preference {
            Preference::Channel(c) => {
                catalog::channel(&c)?;
                next.preferences.channel = c;
            }
            Preference::Language(l) => next.preferences.language = l,
            Preference::Mode(m) => next.preferences.mode = m,
            Preference::Crop(c) => {
                if let Some(c) = &c {
                    c.validate()?;
                    if !self.store.path(&c.source)?.is_file() {
                        return Err(Error::Invalid("Crop source is unavailable".into()));
                    }
                }
                next.preferences.crop = c;
            }
            Preference::Interval(seconds) => {
                if seconds < 10 {
                    return Err(Error::Invalid(
                        "interval must be at least 10 seconds".into(),
                    ));
                }
                next.preferences.interval_seconds = seconds;
            }
            Preference::Retention(images) => {
                if images == 0 {
                    return Err(Error::Invalid(
                        "retention must keep at least one image".into(),
                    ));
                }
                next.preferences.retention = images;
            }
        }
        let retention_changed = next.preferences.retention != self.archive.preferences.retention;
        self.store.save(&next)?;
        self.archive = next;
        if retention_changed {
            self.enforce_retention()?;
        }
        Ok(())
    }
    fn enforce_retention(&mut self) -> Result<()> {
        // Unknown active wallpaper means cleanup is unsafe, especially with multiple monitors.
        let Some(active) = wallpaper::active() else {
            return Ok(());
        };
        let active_name = active
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        self.enforce_retention_for_active(active_name)
    }
    fn enforce_retention_for_active(&mut self, active_name: &str) -> Result<()> {
        let candidates = lifecycle::retention_candidates(&self.archive, Some(active_name));
        // A malformed or legacy manifest may have multiple records for one file.
        // Never remove it when any reference is outside the candidate set.
        let candidates: Vec<_> = candidates
            .into_iter()
            .filter(|file| {
                self.archive
                    .records
                    .iter()
                    .filter(|record| &record.file == file)
                    .count()
                    == 1
            })
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        let mut next = self.archive.clone();
        next.records
            .retain(|record| !candidates.contains(&record.file));
        // Commit the manifest first. Failure while deleting leaves only harmless orphan files.
        self.store.save(&next)?;
        self.archive = next;
        for file in candidates {
            std::fs::remove_file(self.store.path(&file)?)?;
        }
        Ok(())
    }
    pub fn acquire(&mut self, channel: &str, conditional: bool, apply: bool) -> Outcome {
        let mut out = Outcome::new(channel);
        let catalog = match catalog::channel(channel) {
            Ok(c) => c,
            Err(e) => {
                out.check = Stage::Failed(e.to_string());
                return out;
            }
        };
        let modified = if conditional {
            self.archive
                .latest(channel)
                .filter(|r| self.store.path(&r.file).is_ok_and(|p| p.is_file()))
                .and_then(|r| r.remote_modified.as_deref())
        } else {
            None
        };
        let fetch = match self.source.fetch(&catalog.url, modified) {
            Ok(f) => f,
            Err(e) => {
                if conditional {
                    out.check = Stage::Failed(e.to_string())
                } else {
                    out.download = Stage::Failed(e.to_string())
                }
                return out;
            }
        };
        if conditional {
            out.check = Stage::Succeeded;
        }
        match fetch {
            Fetch::Unchanged { modified } => {
                out.modified = modified;
                out.check = Stage::Unchanged;
            }
            Fetch::Downloaded { modified, bytes } => {
                out.modified = modified.clone();
                let image = match decode(&bytes) {
                    Ok(i) => i,
                    Err(e) => {
                        out.download = Stage::Failed(e.to_string());
                        return out;
                    }
                };
                out.download = Stage::Succeeded;
                let id = unique_id();
                let record = Record {
                    id: id.clone(),
                    channel: channel.into(),
                    url: catalog.url.clone(),
                    remote_modified: modified,
                    downloaded_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    bytes: bytes.len() as u64,
                    file: format!("{channel}-{id}.jpg"),
                    width: image.width(),
                    height: image.height(),
                };
                if let Err(e) = self.store.add(&mut self.archive, record.clone(), &bytes) {
                    out.persistence = Stage::Failed(e.to_string());
                    return out;
                }
                out.persistence = Stage::Succeeded;
                if apply {
                    out.apply = stage(self.apply_record(&record));
                }
                if let Err(e) = self.enforce_retention() {
                    eprintln!("Retention cleanup failed: {e}");
                }
            }
        }
        out
    }
    pub fn apply_local(&mut self, id: &str) -> Result<()> {
        let record = self
            .archive
            .records
            .iter()
            .find(|r| r.id == id)
            .cloned()
            .ok_or_else(|| Error::Invalid("Unknown local image".into()))?;
        self.apply_record(&record)
    }
    fn apply_record(&mut self, record: &Record) -> Result<()> {
        // Selection is committed even when the platform adapter is unavailable.
        self.preference(Preference::Channel(record.channel.clone()))?;
        let path = if self.archive.preferences.mode == Mode::SolarCrop {
            let crop =
                self.archive.preferences.crop.as_ref().ok_or_else(|| {
                    Error::Invalid("Solar Crop requires a saved rectangle".into())
                })?;
            self.store.derive(record, crop)?
        } else {
            self.store.path(&record.file)?
        };
        if !path.is_file() {
            return Err(Error::Invalid("Local source is unavailable".into()));
        }
        wallpaper::apply(&path, self.archive.preferences.mode)
    }
}
fn stage(result: Result<()>) -> Stage {
    match result {
        Ok(()) => Stage::Succeeded,
        Err(e) => Stage::Failed(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct Offline;
    impl ImageSource for Offline {
        fn fetch(&self, _: &str, _: Option<&str>) -> Result<Fetch> {
            Err(Error::Invalid("offline test source".into()))
        }
    }

    struct Source {
        result: Mutex<Option<Fetch>>,
        conditional: Mutex<Vec<Option<String>>>,
    }
    impl ImageSource for Source {
        fn fetch(&self, _: &str, modified: Option<&str>) -> Result<Fetch> {
            self.conditional
                .lock()
                .unwrap()
                .push(modified.map(str::to_owned));
            self.result
                .lock()
                .unwrap()
                .take()
                .ok_or_else(|| Error::Invalid("No test response".into()))
        }
    }

    #[test]
    fn download_persists_and_conditional_check_uses_local_timestamp() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().to_path_buf()).unwrap();
        let mut data = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(3, 2)
            .write_to(&mut data, image::ImageFormat::Jpeg)
            .unwrap();
        let source = Source {
            result: Mutex::new(Some(Fetch::Downloaded {
                modified: Some("Wed, 11 Nov 2026 11:34:06 GMT".into()),
                bytes: data.into_inner(),
            })),
            conditional: Mutex::new(Vec::new()),
        };
        let mut service = Service::open(store, source).unwrap();
        let out = service.acquire("0304", false, false);
        assert_eq!(out.persistence, Stage::Succeeded);
        assert_eq!(service.archive.records.len(), 1);
        assert!(
            service
                .store
                .path(&service.archive.records[0].file)
                .unwrap()
                .is_file()
        );
        *service.source.result.lock().unwrap() = Some(Fetch::Unchanged { modified: None });
        let out = service.acquire("0304", true, false);
        assert_eq!(out.check, Stage::Unchanged);
        assert_eq!(service.archive.records.len(), 1);
        assert_eq!(
            service.source.conditional.lock().unwrap()[1].as_deref(),
            Some("Wed, 11 Nov 2026 11:34:06 GMT")
        );
        drop(service);
        assert_eq!(
            Store::open(dir.path().to_path_buf())
                .unwrap()
                .load()
                .unwrap()
                .records
                .len(),
            1
        );
    }

    #[test]
    fn settings_persist_and_invalid_values_do_not_replace_them() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().to_path_buf()).unwrap();
        let mut service = Service::open(store, Offline).unwrap();
        service.preference(Preference::Interval(60)).unwrap();
        service.preference(Preference::Mode(Mode::Center)).unwrap();
        service
            .preference(Preference::Language(Language::Es))
            .unwrap();
        assert!(service.preference(Preference::Interval(1)).is_err());
        drop(service);
        let store = Store::open(dir.path().to_path_buf()).unwrap();
        let archive = store.load().unwrap();
        assert_eq!(archive.preferences.interval_seconds, 60);
        assert_eq!(archive.preferences.mode, Mode::Center);
        assert_eq!(archive.preferences.language, Language::Es);
    }

    #[test]
    fn retention_preserves_active_file_and_removes_obsolete_records() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().to_path_buf()).unwrap();
        let mut service = Service::open(store, Offline).unwrap();
        service.archive.preferences.retention = 1;
        for (id, time) in [("new", 3), ("active", 2), ("old", 1)] {
            let file = format!("{id}.jpg");
            std::fs::write(dir.path().join(&file), [1u8]).unwrap();
            service.archive.records.push(Record {
                id: id.into(),
                channel: "0304".into(),
                url: String::new(),
                remote_modified: None,
                downloaded_at: time,
                bytes: 1,
                file,
                width: 1,
                height: 1,
            });
        }
        service.store.save(&service.archive).unwrap();
        service.enforce_retention_for_active("active.jpg").unwrap();
        assert!(dir.path().join("new.jpg").exists());
        assert!(dir.path().join("active.jpg").exists());
        assert!(!dir.path().join("old.jpg").exists());
        assert_eq!(service.store.load().unwrap().records.len(), 2);
    }
}

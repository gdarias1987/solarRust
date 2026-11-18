use super::Archive;

/// None means unknown, not "no wallpaper": no destructive cleanup is safe.
pub fn retention_candidates(archive: &Archive, active: Option<&str>) -> Vec<String> {
    let Some(active) = active else {
        return Vec::new();
    };
    let mut records: Vec<_> = archive.records.iter().collect();
    records.sort_by_key(|r| std::cmp::Reverse(r.downloaded_at));
    let mut counts = std::collections::HashMap::new();
    records
        .into_iter()
        .filter_map(|r| {
            let n = counts.entry(&r.channel).or_insert(0);
            *n += 1;
            let protected = r.file == active
                || archive
                    .preferences
                    .crop
                    .as_ref()
                    .is_some_and(|c| c.source == r.file);
            (*n > archive.preferences.retention && !protected).then(|| r.file.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Record, crop::Crop};

    fn record(id: &str, channel: &str, time: u64) -> Record {
        Record {
            id: id.into(),
            channel: channel.into(),
            url: String::new(),
            remote_modified: None,
            downloaded_at: time,
            bytes: 1,
            file: format!("{id}.jpg"),
            width: 100,
            height: 100,
        }
    }

    #[test]
    fn protects_unknown_active_wallpaper_and_crop_source() {
        let mut archive = Archive::default();
        archive.preferences.retention = 1;
        archive.records = vec![
            record("new", "0304", 4),
            record("active", "0304", 3),
            record("crop", "0304", 2),
            record("old", "0304", 1),
            record("other", "0193", 1),
        ];
        archive.preferences.crop = Some(Crop::centered("crop.jpg".into(), 100, 100, 1.0).unwrap());
        assert!(retention_candidates(&archive, None).is_empty());
        assert_eq!(
            retention_candidates(&archive, Some("active.jpg")),
            vec!["old.jpg"]
        );
    }
}

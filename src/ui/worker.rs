use rune::{Pixels, Waker};
use solar_rust::{
    application::{ClearHistorySummary, Outcome, Preference, Service},
    domain::Archive,
    infrastructure::{
        nasa::Nasa,
        storage::{Store, data_directory, preview},
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};

pub enum Command {
    Preference(Preference),
    TrackedPreference {
        request_id: u64,
        preference: Preference,
    },
    Acquire {
        conditional: bool,
        apply: bool,
        request_id: Option<u64>,
    },
    Apply {
        id: String,
        request_id: u64,
    },
    ClearHistory,
}
pub struct Snapshot {
    pub archive: Archive,
    pub previews: HashMap<String, Pixels>,
    pub outcome: Option<Outcome>,
    pub error: Option<String>,
}
pub enum Event {
    Snapshot(Box<Snapshot>),
    PreferenceResult {
        request_id: u64,
        error: Option<String>,
    },
    ApplyResult {
        request_id: u64,
        error: Option<String>,
    },
    HistoryClearResult {
        result: std::result::Result<ClearHistorySummary, String>,
    },
    HistoryClearStarted,
    Busy,
    Fatal(String),
}

fn retain_previews<T>(cache: &mut HashMap<String, T>, ids: &HashSet<&str>) {
    cache.retain(|id, _| ids.contains(id.as_str()));
}

fn next_due(now: Instant, interval_seconds: u64) -> Instant {
    now.checked_add(Duration::from_secs(interval_seconds))
        .unwrap_or_else(|| now + Duration::from_secs(365 * 24 * 60 * 60))
}

pub fn start(wake: Waker) -> (Sender<Command>, Receiver<Event>) {
    let (commands, receive) = mpsc::channel();
    let (send, events) = mpsc::channel();
    std::thread::spawn(move || {
        let notify = |event| {
            let _ = send.send(event);
            wake();
        };
        let service =
            Store::open(data_directory()).and_then(|store| Service::open(store, Nasa::new()?));
        let mut service = match service {
            Ok(s) => s,
            Err(e) => {
                notify(Event::Fatal(e.to_string()));
                return;
            }
        };
        let offline = std::env::var_os("SOLAR_OFFLINE").is_some_and(|v| v == "1");
        let mut cache = HashMap::new();
        let mut publish = |service: &Service<Nasa>,
                           outcome: Option<Outcome>,
                           error: Option<String>| {
            let mut records: Vec<_> = service.archive.records.iter().collect();
            records.sort_by_key(|r| std::cmp::Reverse(r.downloaded_at));
            records.truncate(50);
            let ids: HashSet<_> = records.iter().map(|record| record.id.as_str()).collect();
            retain_previews(&mut cache, &ids);
            notify(Event::Snapshot(Box::new(Snapshot {
                archive: service.archive.clone(),
                previews: cache.clone(),
                outcome: outcome.clone(),
                error: error.clone(),
            })));
            for record in records {
                if cache.contains_key(&record.id) {
                    continue;
                }
                match service.store.path(&record.file).and_then(|p| preview(&p)) {
                    Ok((w, h, rgba)) => {
                        let pixels = Pixels::new();
                        pixels.set(w, h, rgba);
                        cache.insert(record.id.clone(), pixels);
                        notify(Event::Snapshot(Box::new(Snapshot {
                            archive: service.archive.clone(),
                            previews: cache.clone(),
                            outcome: outcome.clone(),
                            error: error.clone(),
                        })));
                    }
                    Err(error) => eprintln!("Unable to load preview for {}: {error}", record.file),
                }
            }
        };
        publish(&service, None, None);
        // Let the initial archive snapshot render before background network work starts. An
        // immediate local wallpaper apply must not sit behind the startup NASA request.
        let mut due = next_due(Instant::now(), service.archive.preferences.interval_seconds);
        loop {
            let command = receive.recv_timeout(if offline {
                Duration::from_secs(3600)
            } else {
                due.saturating_duration_since(Instant::now())
            });
            if offline && matches!(&command, Err(mpsc::RecvTimeoutError::Timeout)) {
                continue;
            }
            let mut outcome = None;
            let mut error = None;
            // A preference write is housekeeping, not a sync: showing "Syncing" for it makes
            // the shell look busy when nothing is being fetched.
            if !matches!(
                command,
                Ok(Command::Preference(_) | Command::TrackedPreference { .. })
            ) {
                notify(Event::Busy);
            }
            let mut tracked_result = None;
            let mut apply_request = None;
            let mut history_clear_result = None;
            match command {
                Ok(Command::Preference(p)) => {
                    let interval_changed = matches!(&p, Preference::Interval(_));
                    match service.preference(p) {
                        Ok(()) if interval_changed => {
                            due = next_due(
                                Instant::now(),
                                service.archive.preferences.interval_seconds,
                            )
                        }
                        Err(e) => error = Some(e.to_string()),
                        _ => {}
                    }
                }
                Ok(Command::TrackedPreference {
                    request_id,
                    preference,
                }) => {
                    let result = service.preference(preference).err().map(|e| e.to_string());
                    error = result.clone();
                    tracked_result = Some((request_id, result));
                }
                Ok(Command::Apply { id, request_id }) => {
                    apply_request = Some(request_id);
                    if let Err(e) = service.apply_local(&id) {
                        error = Some(e.to_string());
                    }
                }
                Ok(Command::ClearHistory) => {
                    notify(Event::HistoryClearStarted);
                    history_clear_result = Some(service.clear_history().map_err(|e| e.to_string()));
                    if let Some(Err(message)) = &history_clear_result {
                        error = Some(message.clone());
                    }
                }
                Ok(Command::Acquire {
                    conditional,
                    apply,
                    request_id,
                }) => {
                    if apply {
                        apply_request = request_id;
                    }
                    if offline {
                        error = Some("Network disabled by SOLAR_OFFLINE=1".into());
                    } else {
                        let channel = service.archive.preferences.channel.clone();
                        outcome = Some(service.acquire(&channel, conditional, apply));
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) if !offline => {
                    let channel = service.archive.preferences.channel.clone();
                    outcome = Some(service.acquire(&channel, true, true));
                    due = next_due(Instant::now(), service.archive.preferences.interval_seconds);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                _ => continue,
            }
            if let Some(o) = &outcome {
                eprintln!("Operation: {o:?}");
            }
            if error.is_none() {
                error = outcome.as_ref().and_then(|o| {
                    [&o.check, &o.download, &o.persistence, &o.apply]
                        .into_iter()
                        .find_map(|stage| match stage {
                            solar_rust::application::Stage::Failed(message) => {
                                Some(message.clone())
                            }
                            _ => None,
                        })
                });
            }
            if let Some(e) = &error {
                eprintln!("Operation error: {e}");
            }
            // Resolve request-specific feedback before decoding/publishing archive previews.
            // A wallpaper can already be set successfully while preview generation for a new
            // capture is still expensive; holding the Apply button in "Applying…" until then
            // makes the operation look stuck even though the OS has completed it.
            if let Some((request_id, error)) = tracked_result {
                notify(Event::PreferenceResult { request_id, error });
            }
            if let Some(request_id) = apply_request {
                notify(Event::ApplyResult {
                    request_id,
                    error: error.clone(),
                });
            }
            publish(&service, outcome, error);
            if let Some(result) = history_clear_result {
                notify(Event::HistoryClearResult { result });
            }
        }
    });
    (commands, events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_cache_discards_records_outside_current_window() {
        let mut cache = HashMap::from([(String::from("kept"), 1), (String::from("removed"), 2)]);
        retain_previews(&mut cache, &HashSet::from(["kept", "new"]));
        assert_eq!(cache, HashMap::from([(String::from("kept"), 1)]));
        retain_previews(&mut cache, &HashSet::new());
        assert!(cache.is_empty());
    }

    #[test]
    fn interval_deadline_is_relative_to_the_change() {
        let now = Instant::now();
        assert_eq!(
            next_due(now, 30).duration_since(now),
            Duration::from_secs(30)
        );
        assert_eq!(
            next_due(now, 300).duration_since(now),
            Duration::from_secs(300)
        );
    }
}

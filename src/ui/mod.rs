//! The Mission Deck: one window, five surfaces, and a demo fixture so the shell can be
//! inspected headlessly (and on a first launch with an empty archive).

mod art;
mod components;
pub(crate) mod effects;
mod icons;
mod reference_previews;
mod theme;
mod views;
mod worker;

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use rune::{Pixels, Tx, Waker};
use solar_rust::{
    application::{Outcome, Preference, Stage},
    domain::{Archive, Language, Mode, Record, catalog, crop::Crop},
};
use worker::{Command, Event};

pub use views::view;

/// The band the app draws instead of Windows' native title bar. `rune::Titlebar` and the
/// titlebar view must agree, or the caption buttons sit off-centre.
pub const TITLEBAR_HEIGHT: f32 = 52.0;

/// PNG bytes for `rune::set_app_icon`. Generated, so the binary has no image asset to ship.
pub fn app_icon() -> Vec<u8> {
    art::app_icon_png(256)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Channels,
    Gallery,
    Detail,
    Settings,
    Statistics,
    About,
}

pub struct State {
    pub archive: Archive,
    pub previews: HashMap<String, Pixels>,
    pub reference_previews: HashMap<String, Pixels>,
    pub page: Page,
    pub selected_image: Option<String>,
    pub draft: Option<Crop>,
    pub outcome: Option<Outcome>,
    pub error: Option<String>,
    pub busy: bool,
    pub clear_history_confirming: bool,
    pub clear_history_pending: bool,
    pub clear_history_queued: bool,
    pub clear_history_feedback: Option<ClearHistoryFeedback>,
    /// Request-specific feedback for wallpaper application (not generic sync activity).
    pub apply_feedback: Option<ApplyFeedback>,
    apply_request: Option<u64>,
    next_apply_request: u64,
    /// Set by the `demo` scene: the worker never starts and fixtures are never persisted.
    pub demo_mode: bool,
    /// Instrument grid filter: 0 all, 1 AIA, 2 HMI, 3 composite.
    pub filter: usize,
    /// Uncommitted wallpaper layout selected on the Settings page.
    appearance_draft: Option<Mode>,
    /// A wallpaper layout write sent to the worker and awaiting its request-specific result.
    appearance_pending: Option<(u64, Mode)>,
    next_appearance_request: u64,
    /// Optimistic channel selection awaiting a worker snapshot that confirms it.
    pending_channel: Option<(u64, String)>,
    /// The pointer's travel at the last crop drag sample, in window points.
    pub crop_drag: Option<(f32, f32)>,
    commands: Option<Sender<Command>>,
    events: Option<Receiver<Event>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyFeedback {
    Applying,
    Applied,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClearHistoryFeedback {
    Cleared {
        preserved_files: usize,
        active_unknown: bool,
    },
    Failed(String),
}

impl Default for State {
    fn default() -> Self {
        Self {
            archive: Archive::default(),
            previews: HashMap::new(),
            reference_previews: reference_previews::load(),
            page: Page::Channels,
            selected_image: None,
            draft: None,
            outcome: None,
            error: None,
            busy: false,
            clear_history_confirming: false,
            clear_history_pending: false,
            clear_history_queued: false,
            clear_history_feedback: None,
            apply_feedback: None,
            apply_request: None,
            next_apply_request: 0,
            demo_mode: false,
            filter: 0,
            appearance_draft: None,
            appearance_pending: None,
            next_appearance_request: 0,
            pending_channel: None,
            crop_drag: None,
            commands: None,
            events: None,
        }
    }
}

impl State {
    pub fn send(&mut self, command: Command) -> bool {
        if let Some(sender) = &self.commands
            && sender.send(command).is_ok()
        {
            self.busy = true;
            self.error = None;
            return true;
        }
        self.error = Some("Worker is unavailable".into());
        false
    }

    pub fn preference(&mut self, preference: Preference) {
        if self.demo_mode {
            self.apply_preference(preference);
        } else {
            self.send(Command::Preference(preference));
        }
    }

    fn apply_preference(&mut self, preference: Preference) {
        match preference {
            Preference::Channel(channel) => {
                self.archive.preferences.channel = channel;
                self.selected_image = None;
            }
            Preference::Language(language) => self.archive.preferences.language = language,
            Preference::Mode(mode) => self.archive.preferences.mode = mode,
            Preference::Crop(crop) => self.archive.preferences.crop = crop,
            Preference::Interval(seconds) => self.archive.preferences.interval_seconds = seconds,
            Preference::Retention(images) => self.archive.preferences.retention = images,
        }
    }

    pub fn appearance_mode(&self) -> Mode {
        self.appearance_draft
            .or_else(|| self.appearance_pending.map(|(_, mode)| mode))
            .unwrap_or(self.archive.preferences.mode)
    }

    pub fn select_appearance(&mut self, mode: Mode) {
        let base = self
            .appearance_pending
            .map(|(_, pending)| pending)
            .unwrap_or(self.archive.preferences.mode);
        self.appearance_draft = (mode != base).then_some(mode);
    }

    pub fn appearance_is_pending(&self) -> bool {
        self.appearance_pending.is_some()
    }

    pub fn appearance_has_draft(&self) -> bool {
        self.appearance_draft.is_some()
    }

    pub fn apply_appearance(&mut self) {
        if self.appearance_pending.is_some() {
            return;
        }
        let Some(mode) = self.appearance_draft else {
            return;
        };
        if self.demo_mode {
            self.apply_preference(Preference::Mode(mode));
            self.appearance_draft = None;
            return;
        }
        self.next_appearance_request = self.next_appearance_request.wrapping_add(1);
        let request_id = self.next_appearance_request;
        if self.send(Command::TrackedPreference {
            request_id,
            preference: Preference::Mode(mode),
        }) {
            self.appearance_pending = Some((request_id, mode));
            self.appearance_draft = None;
        }
    }

    pub fn cancel_appearance(&mut self) {
        self.appearance_draft = None;
    }

    pub fn image(&self) -> Option<&Record> {
        self.selected_image
            .as_ref()
            .and_then(|id| self.archive.records.iter().find(|r| &r.id == id))
            .or_else(|| self.archive.latest(self.selected_channel()))
    }

    pub fn selected_channel(&self) -> &str {
        self.pending_channel
            .as_ref()
            .map(|(_, channel)| channel.as_str())
            .unwrap_or(&self.archive.preferences.channel)
    }

    pub fn go(&mut self, page: Page) {
        self.page = page;
    }

    pub fn request_clear_history(&mut self) {
        if self.clear_history_pending || self.archive.records.is_empty() {
            return;
        }
        self.clear_history_feedback = None;
        self.clear_history_confirming = true;
    }

    pub fn cancel_clear_history(&mut self) {
        if !self.clear_history_pending {
            self.clear_history_confirming = false;
        }
    }

    pub fn confirm_clear_history(&mut self) {
        if !self.clear_history_confirming || self.clear_history_pending {
            return;
        }
        if self.demo_mode {
            let preserved_files = self.archive.preferences.crop.is_some() as usize;
            self.archive.records.clear();
            self.previews.clear();
            self.selected_image = None;
            self.clear_history_confirming = false;
            self.clear_history_queued = false;
            self.clear_history_feedback = Some(ClearHistoryFeedback::Cleared {
                preserved_files,
                active_unknown: false,
            });
            return;
        }
        let queued = self.busy;
        if self.send(Command::ClearHistory) {
            self.clear_history_pending = true;
            self.clear_history_queued = queued;
            self.clear_history_confirming = false;
        }
    }

    pub fn select_channel(&mut self, id: String) {
        self.selected_image = None;
        self.page = Page::Channels;
        if self.demo_mode {
            self.archive.preferences.channel = id;
            self.pending_channel = None;
        } else {
            self.next_appearance_request = self.next_appearance_request.wrapping_add(1);
            let request_id = self.next_appearance_request;
            if self.send(Command::TrackedPreference {
                request_id,
                preference: Preference::Channel(id.clone()),
            }) {
                self.pending_channel = Some((request_id, id));
            } else {
                self.pending_channel = None;
            }
        }
    }

    pub fn select_image(&mut self, id: String) {
        self.selected_image = Some(id);
        self.page = Page::Detail;
    }

    pub fn acquire(&mut self, conditional: bool, apply: bool) {
        if self.demo_mode {
            let mut outcome = Outcome::new(&self.archive.preferences.channel);
            outcome.check = Stage::Unchanged;
            outcome.modified = Some("fixture".into());
            self.outcome = Some(outcome);
            return;
        }
        let request_id = apply.then(|| self.reserve_apply_request());
        if self.send(Command::Acquire {
            conditional,
            apply,
            request_id,
        }) && let Some(id) = request_id
        {
            self.apply_request = Some(id);
            self.apply_feedback = Some(ApplyFeedback::Applying);
        }
    }

    fn reserve_apply_request(&mut self) -> u64 {
        self.next_apply_request = self.next_apply_request.wrapping_add(1);
        self.next_apply_request
    }

    pub fn apply_selected(&mut self) {
        if self.apply_request.is_some() {
            return;
        }
        let Some(record) = self.image().cloned() else {
            // With no local capture, fetch the selected channel and apply it on download.
            let request_id = self.reserve_apply_request();
            if self.send(Command::Acquire {
                conditional: false,
                apply: true,
                request_id: Some(request_id),
            }) {
                self.apply_request = Some(request_id);
                self.apply_feedback = Some(ApplyFeedback::Applying);
            }
            return;
        };
        if self.demo_mode {
            let mut outcome = Outcome::new(&record.channel);
            outcome.check = Stage::Unchanged;
            outcome.download = Stage::Unchanged;
            outcome.persistence = Stage::Succeeded;
            outcome.apply =
                Stage::Failed("Wallpaper adapter is not implemented on this platform".into());
            self.outcome = Some(outcome);
            self.error = Some("Wallpaper adapter is not implemented on this platform".into());
            return;
        }
        let request_id = self.reserve_apply_request();
        if self.send(Command::Apply {
            id: record.id,
            request_id,
        }) {
            self.apply_request = Some(request_id);
            self.apply_feedback = Some(ApplyFeedback::Applying);
        }
    }

    pub fn edit_crop(&mut self) {
        let source = self
            .archive
            .preferences
            .crop
            .as_ref()
            .and_then(|c| self.archive.records.iter().find(|r| r.file == c.source))
            .or_else(|| self.image())
            .cloned();
        let Some(record) = source else {
            self.error = Some("No usable original image for crop editing".into());
            return;
        };
        self.selected_image = Some(record.id.clone());
        self.draft = self.archive.preferences.crop.clone().or_else(|| {
            Crop::centered(record.file.clone(), record.width, record.height, 16.0 / 9.0).ok()
        });
        self.crop_drag = None;
        if self.draft.is_none() {
            self.error = Some("Could not build a crop for this image".into());
        }
    }

    pub fn drag_crop(&mut self, dx: f64, dy: f64) {
        if let Some(crop) = &mut self.draft {
            crop.adjust(dx, dy, 1.0);
        }
    }

    pub fn zoom_crop(&mut self, factor: f64) {
        if let Some(crop) = &mut self.draft {
            crop.adjust(0.0, 0.0, factor);
        }
    }

    pub fn commit_crop(&mut self) {
        let Some(crop) = self.draft.clone() else {
            return;
        };
        if let Err(error) = crop.validate() {
            self.error = Some(error.to_string());
            return;
        }
        self.archive.preferences.crop = Some(crop.clone());
        self.preference(Preference::Crop(Some(crop)));
        self.draft = None;
        self.crop_drag = None;
    }

    pub fn cancel_crop(&mut self) {
        self.draft = None;
        self.crop_drag = None;
    }
}

pub fn setup(state: &mut State, wake: Waker) {
    if state.demo_mode {
        return;
    }
    let (commands, events) = worker::start(wake);
    state.commands = Some(commands);
    state.events = Some(events);
}

fn handle_event(state: &mut State, event: Event) {
    match event {
        Event::Busy => state.busy = true,
        Event::Fatal(error) => {
            state.error = Some(error.clone());
            state.busy = false;
            state.apply_request = None;
            state.apply_feedback = None;
            state.pending_channel = None;
            if state.clear_history_pending {
                state.clear_history_pending = false;
                state.clear_history_queued = false;
                state.clear_history_feedback = Some(ClearHistoryFeedback::Failed(error.clone()));
            }
            if let Some((_, pending_mode)) = state.appearance_pending.take()
                && state.appearance_draft.is_none()
            {
                state.appearance_draft = Some(pending_mode);
            }
        }
        Event::Snapshot(snapshot) => {
            state.archive = snapshot.archive;
            if state
                .selected_image
                .as_ref()
                .is_some_and(|id| !state.archive.records.iter().any(|record| &record.id == id))
            {
                state.selected_image = None;
            }
            if let Some((_, channel)) = &state.pending_channel
                && state.archive.preferences.channel == *channel
            {
                state.pending_channel = None;
            }
            state.previews = snapshot.previews;
            state.outcome = snapshot.outcome;
            state.error = snapshot.error;
            state.busy = false;
        }
        Event::PreferenceResult { request_id, error } => {
            if let Some((pending_id, pending_mode)) = state.appearance_pending
                && request_id == pending_id
            {
                state.appearance_pending = None;
                if let Some(error) = &error {
                    state.error = Some(error.clone());
                    if state.appearance_draft.is_none() {
                        state.appearance_draft = Some(pending_mode);
                    }
                }
            }
            if state
                .pending_channel
                .as_ref()
                .is_some_and(|(pending_id, _)| *pending_id == request_id)
            {
                state.pending_channel = None;
                if let Some(error) = error {
                    state.error = Some(error);
                }
            }
        }
        Event::ApplyResult { request_id, error } => {
            if state.apply_request == Some(request_id) {
                state.apply_request = None;
                state.busy = false;
                if let Some(error) = error {
                    state.error = Some(error);
                    state.apply_feedback = None;
                } else {
                    state.error = None;
                    state.apply_feedback = Some(ApplyFeedback::Applied);
                }
            }
        }
        Event::HistoryClearStarted => {
            state.busy = true;
            state.clear_history_queued = false;
        }
        Event::HistoryClearResult { result } => {
            state.clear_history_pending = false;
            state.clear_history_queued = false;
            state.clear_history_confirming = false;
            state.clear_history_feedback = Some(match result {
                Ok(summary) => ClearHistoryFeedback::Cleared {
                    preserved_files: summary.preserved_files,
                    active_unknown: summary.active_unknown,
                },
                Err(error) => ClearHistoryFeedback::Failed(error),
            });
        }
    }
}

pub fn poll(state: &mut State, _: f64, _: &mut Tx) -> bool {
    if state.demo_mode {
        return false;
    }
    let mut changed = false;
    loop {
        let event = state
            .events
            .as_ref()
            .and_then(|events| events.try_recv().ok());
        let Some(event) = event else {
            break;
        };
        changed = true;
        handle_event(state, event);
    }
    changed
}

/// Deterministic visual fixture: procedural imagery, not NASA data, never persisted.
pub fn demo(state: &mut State) {
    const BASE: u64 = 1_794_355_200; // 2026-11-10 00:00 UTC
    const FIXTURES: [(&str, u64); 6] = [
        ("0304", BASE),
        ("0193", BASE - 5_400),
        ("0171", BASE - 10_800),
        ("0211", BASE - 16_200),
        ("HMIB", BASE - 21_600),
        ("COMP304211171", BASE - 27_000),
    ];

    state.demo_mode = true;
    state.commands = None;
    state.events = None;
    state.error = None;
    state.busy = false;
    state.apply_feedback = None;
    state.apply_request = None;
    state.filter = 0;
    state.appearance_draft = None;
    state.appearance_pending = None;
    state.pending_channel = None;
    state.archive.preferences.language = Language::En;
    state.archive.preferences.channel = "0304".into();
    state.archive.preferences.mode = solar_rust::domain::Mode::SolarCrop;
    state.archive.preferences.interval_seconds = 300;
    state.archive.preferences.retention = 10;
    state.archive.preferences.crop = None;
    state.archive.records.clear();
    state.previews.clear();

    for (index, channel) in catalog::channels().iter().enumerate() {
        let pixels = Pixels::new();
        pixels.set(
            192,
            192,
            art::sun_rgba(192, index as u32 * 37 + 11, &art::palette(&channel.id)),
        );
        state.previews.insert(format!("ch:{}", channel.id), pixels);
    }

    for (index, (channel, at)) in FIXTURES.iter().enumerate() {
        let id = format!("fixture-{index}");
        let pixels = Pixels::new();
        pixels.set(
            384,
            384,
            art::sun_rgba(384, index as u32 * 613 + 7, &art::palette(channel)),
        );
        state.previews.insert(id.clone(), pixels);
        state.archive.records.push(Record {
            id: id.clone(),
            channel: (*channel).into(),
            url: "Synthetic fixture, not NASA imagery".into(),
            remote_modified: None,
            downloaded_at: *at,
            bytes: 2_880_000 + index as u64 * 143_000,
            file: format!("{channel}-{id}.jpg"),
            width: 4096,
            height: 4096,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    fn snapshot(mode: Mode, error: Option<&str>) -> Event {
        channel_snapshot("0304", mode, error)
    }

    fn channel_snapshot(channel: &str, mode: Mode, error: Option<&str>) -> Event {
        let mut archive = Archive::default();
        archive.preferences.channel = channel.into();
        archive.preferences.mode = mode;
        Event::Snapshot(Box::new(worker::Snapshot {
            archive,
            previews: HashMap::new(),
            outcome: None,
            error: error.map(str::to_owned),
        }))
    }

    fn appearance_command(receive: &mpsc::Receiver<Command>) -> (u64, Mode) {
        match receive.try_recv().expect("appearance command") {
            Command::TrackedPreference {
                request_id,
                preference: Preference::Mode(mode),
            } => (request_id, mode),
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn appearance_selection_is_local_until_apply() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        let persisted = state.archive.preferences.mode;

        state.select_appearance(Mode::Span);

        assert_eq!(state.appearance_mode(), Mode::Span);
        assert_eq!(state.archive.preferences.mode, persisted);
        assert!(matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)));

        state.apply_appearance();

        assert_eq!(appearance_command(&receive), (1, Mode::Span));
        assert_eq!(state.archive.preferences.mode, persisted);
        state.apply_appearance();
        assert!(matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn cancelling_appearance_discards_the_draft_without_a_command() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        let persisted = state.archive.preferences.mode;

        state.select_appearance(Mode::Span);
        state.cancel_appearance();

        assert_eq!(state.appearance_mode(), persisted);
        assert_eq!(state.archive.preferences.mode, persisted);
        assert!(matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn stale_snapshot_does_not_release_the_appearance_request() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };

        state.select_appearance(Mode::Span);
        state.apply_appearance();
        assert_eq!(appearance_command(&receive), (1, Mode::Span));

        handle_event(&mut state, snapshot(Mode::Fit, None));
        assert!(state.appearance_is_pending());
        assert_eq!(state.appearance_mode(), Mode::Span);

        state.apply_appearance();
        assert!(matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn stale_channel_snapshot_does_not_reset_pending_appearance_mode() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };

        state.select_appearance(Mode::Span);
        state.apply_appearance();
        let (request_id, _) = appearance_command(&receive);
        state.select_channel("0193".into());
        let channel_request = match receive.try_recv().expect("channel preference") {
            Command::TrackedPreference {
                request_id,
                preference: Preference::Channel(channel),
            } => {
                assert_eq!(channel, "0193");
                request_id
            }
            _ => panic!("unexpected command"),
        };

        handle_event(&mut state, snapshot(Mode::Fit, None));

        assert_eq!(state.archive.preferences.channel, "0304");
        assert_eq!(state.selected_channel(), "0193");
        assert_eq!(state.archive.preferences.mode, Mode::Fit);
        assert_eq!(state.appearance_mode(), Mode::Span);

        handle_event(&mut state, channel_snapshot("0193", Mode::Span, None));
        handle_event(
            &mut state,
            Event::PreferenceResult {
                request_id: channel_request,
                error: None,
            },
        );
        handle_event(
            &mut state,
            Event::PreferenceResult {
                request_id,
                error: None,
            },
        );
        assert_eq!(state.archive.preferences.mode, Mode::Span);
        assert!(state.pending_channel.is_none());
    }

    #[test]
    fn rapid_channel_selections_survive_stale_snapshots_until_latest_is_confirmed() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };

        state.select_channel("0193".into());
        state.select_channel("0171".into());
        assert!(
            matches!(receive.try_recv(), Ok(Command::TrackedPreference { preference: Preference::Channel(id), .. }) if id == "0193")
        );
        let latest_request = match receive.try_recv().expect("latest channel preference") {
            Command::TrackedPreference {
                request_id,
                preference: Preference::Channel(id),
            } => {
                assert_eq!(id, "0171");
                request_id
            }
            _ => panic!("unexpected command"),
        };

        handle_event(&mut state, channel_snapshot("0304", Mode::Fit, None));
        assert_eq!(state.archive.preferences.channel, "0304");
        assert_eq!(state.selected_channel(), "0171");
        assert_eq!(
            state
                .pending_channel
                .as_ref()
                .map(|(_, channel)| channel.as_str()),
            Some("0171")
        );

        handle_event(&mut state, channel_snapshot("0193", Mode::Fit, None));
        assert_eq!(state.archive.preferences.channel, "0193");
        assert_eq!(state.selected_channel(), "0171");
        assert_eq!(
            state
                .pending_channel
                .as_ref()
                .map(|(_, channel)| channel.as_str()),
            Some("0171")
        );

        handle_event(&mut state, channel_snapshot("0171", Mode::Fit, None));
        assert_eq!(state.archive.preferences.channel, "0171");
        assert!(state.pending_channel.is_none());
        handle_event(
            &mut state,
            Event::PreferenceResult {
                request_id: latest_request,
                error: None,
            },
        );
    }

    #[test]
    fn failed_channel_persistence_snapshot_and_result_roll_back_optimistic_selection() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        state.select_channel("0193".into());
        let request_id = match receive.try_recv().unwrap() {
            Command::TrackedPreference { request_id, .. } => request_id,
            _ => panic!("expected tracked preference"),
        };
        assert_eq!(state.selected_channel(), "0193");

        handle_event(
            &mut state,
            channel_snapshot("0304", Mode::Fit, Some("disk full")),
        );
        assert_eq!(state.archive.preferences.channel, "0304");
        assert_eq!(state.selected_channel(), "0193");
        assert!(state.pending_channel.is_some());
        handle_event(
            &mut state,
            Event::PreferenceResult {
                request_id,
                error: Some("disk full".into()),
            },
        );

        assert!(state.pending_channel.is_none());
        assert_eq!(state.archive.preferences.channel, "0304");
        assert_eq!(state.selected_channel(), "0304");
    }

    #[test]
    fn failed_mode_write_then_cancel_returns_to_persisted_mode() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        let previous = state.archive.preferences.mode;
        state.select_appearance(Mode::Span);
        state.apply_appearance();
        let (request_id, _) = appearance_command(&receive);

        handle_event(&mut state, snapshot(previous, Some("disk full")));
        assert_eq!(state.archive.preferences.mode, previous);
        assert_eq!(state.appearance_mode(), Mode::Span);
        handle_event(
            &mut state,
            Event::PreferenceResult {
                request_id,
                error: Some("disk full".into()),
            },
        );
        assert!(state.appearance_has_draft());

        state.cancel_appearance();
        assert_eq!(state.archive.preferences.mode, previous);
        assert_eq!(state.appearance_mode(), previous);
    }

    #[test]
    fn failed_channel_send_clears_pending_and_restores_previous_selection() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        state.select_channel("0193".into());
        assert_eq!(state.selected_channel(), "0193");
        drop(receive);

        state.select_channel("0171".into());

        assert!(state.pending_channel.is_none());
        assert_eq!(state.archive.preferences.channel, "0304");
        assert_eq!(state.selected_channel(), "0304");
        assert_eq!(state.error.as_deref(), Some("Worker is unavailable"));
    }

    #[test]
    fn fatal_worker_event_clears_pending_channel() {
        let (commands, _receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        state.select_channel("0193".into());
        assert_eq!(state.selected_channel(), "0193");

        handle_event(&mut state, Event::Fatal("storage unavailable".into()));

        assert!(state.pending_channel.is_none());
        assert_eq!(state.selected_channel(), "0304");
        assert_eq!(state.error.as_deref(), Some("storage unavailable"));
    }

    #[test]
    fn apply_selected_dispatches_local_apply_for_the_latest_capture() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        state.archive.records.push(Record {
            id: "capture-1".into(),
            channel: "0304".into(),
            url: String::new(),
            remote_modified: None,
            downloaded_at: 1,
            bytes: 1,
            file: "capture.jpg".into(),
            width: 1,
            height: 1,
        });

        state.apply_selected();

        let request_id = match receive.try_recv() {
            Ok(Command::Apply { id, request_id }) if id == "capture-1" => request_id,
            _ => panic!("unexpected command"),
        };
        assert_eq!(state.apply_feedback, Some(ApplyFeedback::Applying));
        assert!(state.busy);

        handle_event(
            &mut state,
            Event::ApplyResult {
                request_id,
                error: None,
            },
        );
        assert_eq!(state.apply_feedback, Some(ApplyFeedback::Applied));
        assert!(!state.busy);
        assert!(state.error.is_none());
    }

    #[test]
    fn failed_apply_clears_pending_feedback_and_exposes_the_error() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        state.archive.records.push(Record {
            id: "capture-1".into(),
            channel: "0304".into(),
            url: String::new(),
            remote_modified: None,
            downloaded_at: 1,
            bytes: 1,
            file: "capture.jpg".into(),
            width: 1,
            height: 1,
        });
        state.apply_selected();
        let request_id = match receive.try_recv().unwrap() {
            Command::Apply { request_id, .. } => request_id,
            _ => unreachable!(),
        };
        handle_event(
            &mut state,
            Event::ApplyResult {
                request_id,
                error: Some("adapter failed".into()),
            },
        );
        assert_eq!(state.apply_feedback, None);
        assert_eq!(state.error.as_deref(), Some("adapter failed"));
        assert!(state.apply_request.is_none());
    }

    #[test]
    fn clear_history_confirmation_can_be_cancelled_without_sending_a_command() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        state.archive.records.push(Record {
            id: "capture-1".into(),
            channel: "0304".into(),
            url: String::new(),
            remote_modified: None,
            downloaded_at: 1,
            bytes: 1,
            file: "capture.jpg".into(),
            width: 1,
            height: 1,
        });

        state.request_clear_history();
        assert!(state.clear_history_confirming);
        state.cancel_clear_history();

        assert!(!state.clear_history_confirming);
        assert_eq!(state.archive.records.len(), 1);
        assert!(matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn clear_history_confirmation_sends_worker_command_only_after_confirm() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        state.archive.records.push(Record {
            id: "capture-1".into(),
            channel: "0304".into(),
            url: String::new(),
            remote_modified: None,
            downloaded_at: 1,
            bytes: 1,
            file: "capture.jpg".into(),
            width: 1,
            height: 1,
        });
        state.request_clear_history();
        assert!(matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)));

        state.confirm_clear_history();

        assert!(matches!(receive.try_recv(), Ok(Command::ClearHistory)));
        assert!(state.clear_history_pending);
        assert!(!state.clear_history_confirming);
    }

    #[test]
    fn clear_history_can_be_queued_while_another_worker_operation_is_busy() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            busy: true,
            ..State::default()
        };
        state.archive.records.push(Record {
            id: "capture-1".into(),
            channel: "0304".into(),
            url: String::new(),
            remote_modified: None,
            downloaded_at: 1,
            bytes: 1,
            file: "capture.jpg".into(),
            width: 1,
            height: 1,
        });

        state.request_clear_history();
        state.confirm_clear_history();

        assert!(matches!(receive.try_recv(), Ok(Command::ClearHistory)));
        assert!(state.clear_history_pending);
        assert!(state.clear_history_queued);

        handle_event(&mut state, Event::HistoryClearStarted);
        assert!(!state.clear_history_queued);
    }

    #[test]
    fn clear_history_result_reports_success_and_failure() {
        let mut state = State {
            clear_history_pending: true,
            ..State::default()
        };
        handle_event(
            &mut state,
            Event::HistoryClearResult {
                result: Ok(solar_rust::application::ClearHistorySummary {
                    removed_records: 2,
                    preserved_files: 1,
                    active_unknown: true,
                }),
            },
        );
        assert!(!state.clear_history_pending);
        assert_eq!(
            state.clear_history_feedback,
            Some(ClearHistoryFeedback::Cleared {
                preserved_files: 1,
                active_unknown: true,
            })
        );

        state.clear_history_pending = true;
        handle_event(
            &mut state,
            Event::HistoryClearResult {
                result: Err("disk full".into()),
            },
        );
        assert!(!state.clear_history_pending);
        assert_eq!(
            state.clear_history_feedback,
            Some(ClearHistoryFeedback::Failed("disk full".into()))
        );
    }

    #[test]
    fn fatal_worker_event_clears_pending_apply_feedback() {
        let (commands, _receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        state.archive.records.push(Record {
            id: "capture-1".into(),
            channel: "0304".into(),
            url: String::new(),
            remote_modified: None,
            downloaded_at: 1,
            bytes: 1,
            file: "capture.jpg".into(),
            width: 1,
            height: 1,
        });
        state.apply_selected();
        assert_eq!(state.apply_feedback, Some(ApplyFeedback::Applying));

        handle_event(&mut state, Event::Fatal("storage unavailable".into()));

        assert_eq!(state.apply_feedback, None);
        assert!(state.apply_request.is_none());
        assert_eq!(state.error.as_deref(), Some("storage unavailable"));
    }

    #[test]
    fn apply_selected_fetches_and_applies_when_no_local_capture_exists() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };

        state.apply_selected();

        assert!(matches!(
            receive.try_recv(),
            Ok(Command::Acquire {
                conditional: false,
                apply: true,
                request_id: Some(_)
            })
        ));
    }

    #[test]
    fn selection_of_old_persisted_mode_survives_pending_acknowledgement() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };

        state.select_appearance(Mode::Span);
        state.apply_appearance();
        let (request_id, _) = appearance_command(&receive);
        state.select_appearance(Mode::Fit);

        handle_event(&mut state, snapshot(Mode::Span, None));
        handle_event(
            &mut state,
            Event::PreferenceResult {
                request_id,
                error: None,
            },
        );

        assert!(!state.appearance_is_pending());
        assert!(state.appearance_has_draft());
        assert_eq!(state.archive.preferences.mode, Mode::Span);
        assert_eq!(state.appearance_mode(), Mode::Fit);
    }

    #[test]
    fn successful_appearance_result_commits_and_clears_pending_state() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };

        state.select_appearance(Mode::Span);
        state.apply_appearance();
        let (request_id, _) = appearance_command(&receive);
        handle_event(&mut state, snapshot(Mode::Span, None));
        handle_event(
            &mut state,
            Event::PreferenceResult {
                request_id,
                error: None,
            },
        );

        assert_eq!(state.archive.preferences.mode, Mode::Span);
        assert_eq!(state.appearance_mode(), Mode::Span);
        assert!(!state.appearance_is_pending());
        assert!(!state.appearance_has_draft());
    }

    #[test]
    fn failed_appearance_result_restores_the_draft_for_retry() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };

        state.select_appearance(Mode::Span);
        state.apply_appearance();
        let (request_id, _) = appearance_command(&receive);
        handle_event(&mut state, snapshot(Mode::Fit, Some("save failed")));
        handle_event(
            &mut state,
            Event::PreferenceResult {
                request_id,
                error: Some("save failed".into()),
            },
        );

        assert!(!state.appearance_is_pending());
        assert_eq!(state.appearance_mode(), Mode::Span);
        assert_eq!(state.error.as_deref(), Some("save failed"));

        state.apply_appearance();
        assert_eq!(appearance_command(&receive), (2, Mode::Span));
    }

    #[test]
    fn fatal_worker_event_restores_pending_mode_without_overwriting_newer_draft() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        state.select_appearance(Mode::Span);
        state.apply_appearance();
        appearance_command(&receive);

        handle_event(&mut state, Event::Fatal("worker failed".into()));

        assert!(!state.appearance_is_pending());
        assert_eq!(state.appearance_mode(), Mode::Span);
        assert_eq!(state.error.as_deref(), Some("worker failed"));

        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        state.select_appearance(Mode::Span);
        state.apply_appearance();
        appearance_command(&receive);
        state.select_appearance(Mode::Fit);

        handle_event(&mut state, Event::Fatal("worker failed".into()));

        assert!(!state.appearance_is_pending());
        assert_eq!(state.appearance_mode(), Mode::Fit);
    }

    #[test]
    fn cancelling_a_new_draft_keeps_the_mode_already_being_saved() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };

        state.select_appearance(Mode::Span);
        state.apply_appearance();
        appearance_command(&receive);
        state.select_appearance(Mode::Fit);
        assert_eq!(state.appearance_mode(), Mode::Fit);

        state.cancel_appearance();

        assert!(state.appearance_is_pending());
        assert_eq!(state.appearance_mode(), Mode::Span);
    }

    #[test]
    fn demo_appearance_apply_is_immediate_and_never_sends_a_command() {
        let (commands, receive) = mpsc::channel();
        let mut state = State {
            demo_mode: true,
            commands: Some(commands),
            ..State::default()
        };

        state.select_appearance(Mode::Span);
        state.apply_appearance();

        assert_eq!(state.archive.preferences.mode, Mode::Span);
        assert_eq!(state.appearance_mode(), Mode::Span);
        assert!(!state.appearance_is_pending());
        assert!(!state.appearance_has_draft());
        assert!(matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }
}

// A GUI app: no terminal window opens on launch, so the taskbar shows the app instead of a
// console host. The test harness stays a console program so `cargo test` still prints.
#![cfg_attr(all(target_os = "windows", not(test)), windows_subsystem = "windows")]

mod ui;

use solar_rust::domain::Language;
use ui::{Page, State};

fn main() {
    attach_parent_console();
    eprintln!(
        "Solar Observatory {} | Rune 38e75af6 | data: {}",
        env!("CARGO_PKG_VERSION"),
        solar_rust::infrastructure::storage::data_directory().display()
    );
    if let Err(error) = rune::set_app_icon(ui::app_icon()) {
        eprintln!("App icon unavailable: {error}");
    }
    rune::App::new(rune::AppSpec {
        min_size: Some((1000.0, 760.0)),
        setup: Some(ui::setup),
        poll: Some(ui::poll),
        families: ("Inter", "Cascadia Mono, Consolas, Menlo, ui-monospace"),
        scenes: &[
            ("demo", ui::demo),
            ("channels", |s: &mut State| {
                ui::demo(s);
                s.page = Page::Channels;
            }),
            ("gallery", |s: &mut State| {
                ui::demo(s);
                s.page = Page::Gallery;
            }),
            ("detail", |s: &mut State| {
                ui::demo(s);
                s.selected_image = s.archive.records.first().map(|r| r.id.clone());
                s.page = Page::Detail;
            }),
            ("settings", |s: &mut State| {
                ui::demo(s);
                s.page = Page::Settings;
            }),
            ("statistics", |s: &mut State| {
                ui::demo(s);
                s.page = Page::Statistics;
            }),
            ("about", |s: &mut State| {
                ui::demo(s);
                s.page = Page::About;
            }),
            ("crop", |s: &mut State| {
                ui::demo(s);
                s.edit_crop();
            }),
            ("spanish", |s: &mut State| {
                ui::demo(s);
                s.archive.preferences.language = Language::Es;
            }),
        ],
        on_key: Some(|s: &mut State, event, _| {
            if event.key == rune::Key::Escape {
                s.draft = None;
                s.crop_drag = None;
                true
            } else {
                false
            }
        }),
        ..rune::AppSpec::new(
            "Solar Observatory",
            (1180.0, 820.0),
            State::default,
            ui::view,
        )
    })
    .titlebar(rune::Titlebar::new(ui::TITLEBAR_HEIGHT))
    .compositor(ui::effects::SolarFx::default())
    .run();
}

/// A GUI-subsystem process starts with no console, which is what keeps a double-click from
/// opening a terminal window. A run from a shell (`cargo run`, scripted snapshots) still expects
/// the banner on that shell, so attach to its console when there is one. No parent console (a
/// double-click) makes the call fail harmlessly and the process stays quiet.
#[cfg(all(target_os = "windows", not(test)))]
fn attach_parent_console() {
    // SAFETY: associates this process with an existing console. No pointer is passed and a
    // failure leaves the process as it was.
    unsafe {
        let _ = windows::Win32::System::Console::AttachConsole(
            windows::Win32::System::Console::ATTACH_PARENT_PROCESS,
        );
    }
}

#[cfg(not(all(target_os = "windows", not(test))))]
fn attach_parent_console() {}

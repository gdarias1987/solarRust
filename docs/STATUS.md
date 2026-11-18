# Solar Observatory — Project Status

Living tracker for `solarRust`, a native desktop port of Solar Go built with
[anomalyco/rune](https://github.com/anomalyco/rune) pinned to
`38e75af6c22915c01d5e219f53d5f0af6edf9a11`.

**Headline:** the application is complete for its stated v1 scope. Build, lint, test, and visual checks are recorded in the verification log below; platform-specific wallpaper behavior still requires validation on real Windows, macOS, and Linux desktop sessions.

Last verified: 2026-11-18.

---

## Snapshot

| Item | State |
|---|---|
| Crate | `solar-rust` 0.1.0, edition 2024 |
| Rune pin | `38e75af6c22915c01d5e219f53d5f0af6edf9a11` |
| Build | `cargo check --locked` → passes, no warnings |
| Lints | `cargo clippy --all-targets -- -D warnings` → clean |
| Tests | See the dated verification log below; rerun before release |
| CI | `.github/workflows/ci.yml` — clippy and tests on `windows-latest`, `macos-latest`, `ubuntu-latest`; headless smoke render on Windows |
| Git | `main`; repository baseline is being prepared for its initial commit |
| Wallpaper | Three backends behind one pair of calls: Windows COM, macOS `NSWorkspace`, Linux desktop tools |

## Architecture

```
src/main.rs            Rune AppSpec, scenes, custom titlebar, generated app icon
src/application.rs     Service<S>: serialized use cases, per-stage Outcome
src/domain/            catalog, crop, lifecycle (retention), Archive/Record/Preferences
src/infrastructure/    nasa (HTTP), storage (filesystem), wallpaper/{windows,macos,linux}
src/ui/                State, worker thread, theme, icons, art, components
src/ui/views/          chrome, channels, gallery, detail, settings, statistics, about, crop
content/               channels.json (19 SDO products), reference previews
```

One worker thread owns `Service`; the UI sends `Command`s and renders `Snapshot`s. That
single-writer shape is load-bearing — keep it.

## Functional coverage

| Requirement | State | Where |
|---|---|---|
| 19 channel identities and explicit composite mappings | Done | `content/channels.json`, `domain::catalog` |
| Selecting a channel persists without applying | Done | `Service::preference(Channel)` |
| Apply downloads and updates the default on success | Done | `Service::apply_record` |
| Scheduled check targets the selected channel, skips unchanged | Done | `worker::start`, `Nasa::fetch` |
| Manual apply distinct from conditional check | Done | `Command::Acquire { conditional, apply }` |
| Apply a stored image without downloading | Done | `Service::apply_local` |
| Mode, language, interval, retention and crop survive restart | Done | `Store::{save, load}` |
| Changing mode alone does not apply | Done | `Preference::Mode` only saves |
| Crop coordinates survive scaling and source changes | Done | `crop::Crop::{scaled, validate}` |
| Solar Crop without a saved rectangle is an error | Done | `Service::apply_record` |
| Retention protects active wallpaper and crop source | Done | `lifecycle::retention_candidates` |
| Download, persistence and apply errors stay distinguishable | Done | `Outcome` stages |
| Gallery, statistics, settings, About | Done | `ui::views::*` |
| Bilingual chrome and catalogue prose | Done | `views::tr`, `Channel::{description, region, simple, kids}` |
| Channel explained in three reading tiers | Done | `views::channels::hero` (technical, plain words, for young astronomers) |
| Temperature in kelvin plus the reader's scale | Done | `views::temperature`, `views::temperature_equivalent` |
| Wallpaper set on Windows, macOS and Linux | Done | `infrastructure::wallpaper::{windows,macos,linux}` |
| Interactive solar-system scene | Not in scope | see Decisions |

## Presentation

- **Chrome** — Rune's own titlebar (`App::titlebar`) replaces the native Windows frame; the
  app pads by `Cx::chrome_insets().trailing` so its controls stop short of Rune's caption
  buttons, and sets `caption_color` so those symbols follow the dark theme.
- **Icon** — generated at startup (`ui::art::app_icon_png`) and installed with
  `rune::set_app_icon`; no image asset ships with the binary.
- **Ground** — three static aurora fields, a technical grid, a vignette, and one wide bloom on
  a 23 s opacity clock so the shell never reads as a dead screenshot.
- **Motion** — hover and press settle on short springs; pages mount with a 0.994 scale and
  6 pt offset; the status light breathes (fast while syncing, slow at rest).
- **Blur budget** — at most four `backdrop_blur` surfaces: titlebar, sidebar, crop overlay.
  `system_material` is macOS-only and is deliberately unused.
- **Reading tiers** — the Channels hero carries the same instrument three times: the catalogue
  line as its subtitle, a jargon-free paragraph, and a violet callout written for a child. The
  panel is content-sized, so long prose grows the deck instead of clipping; below 1180×820 the
  page scrolls and fades its bottom edge rather than cutting the instrument grid.
- **Instrument grid** — the nineteen channels are a rack of square tiles, not a carousel. The
  column count comes from the measured width of `channels-body`, so the page shows five columns
  at 1180 wide and eight at 1600. Rows wrap, the last row stays left-aligned, and the grid only
  grows downwards: no surface in the app scrolls sideways.

## Known gaps

| Severity | Gap | Note |
|---|---|---|
| M | No virtualisation in the archive grid | Previews are capped at 50 in the worker; records are not paginated |
| M | One `Error` enum covers domain, IO, HTTP, image and platform | Adequate today; split when a second consumer appears |
| M | Linux anchors are untested on real sessions | Detection, URI encoding and every command plan are unit-tested, and the file typechecks on every host, but no GNOME/KDE/Sway session has run it |
| L | KDE gets no fill mode and no read-back | `plasma-apply-wallpaperimage` takes an image only; the crop mode stays whatever Plasma already uses, and retention sees "unknown" |
| L | Hyprland needs `hyprpaper` running | `hyprctl hyprpaper wallpaper` fails cleanly when the plugin is absent; no fallback ships |
| L | Two `objc2` versions coexist on macOS (0.5 + 0.6) | Predates this change and costs nothing to the backend: `winit`/`accesskit_macos` pin 0.5/0.2, while `wgpu-hal`, `fontique` and `arboard` — all reached through Rune — already pull 0.6/0.3, so the wallpaper code added no crate, only extra modules inside ones already compiled |
| L | i18n lives in `domain` | `Channel` carries its own translations; fine while there are two languages |
| L | Preview cache is bounded by record count, not bytes | 50 × 640 px RGBA worst case; revisit if channel counts grow |
| L | The taskbar itself could not be captured in this session | It is auto-hidden and `PrintWindow` on `Shell_TrayWnd` returns black, so the icon was verified from the resource and the window's `HICON` bitmaps instead of a screenshot |

## Decisions

| Topic | Decision |
|---|---|
| Presentation | Rune `El` primitives plus `rune-kit` for controls; no second UI framework |
| Window frame | App-drawn titlebar on every platform, so Windows loses the native bar and gains themed caption buttons |
| Glass | `backdrop_blur` rather than `system_material`; portable and cheap at this element count |
| Wallpaper | One `apply`/`active` pair per system: Windows `IDesktopWallpaper`, macOS `NSWorkspace` per screen, Linux the desktop's own tool. `active() == None` means "unknown", and retention then deletes nothing |
| macOS threading | Wallpaper work is dispatched to the main queue with a 2 s budget, because `NSScreen` requires it; a headless session fails instead of hanging |
| macOS fallback | System Events runs only when the native call or its read-back fails, so the Automation prompt is never triggered on a healthy system |
| App identity | The Sun is drawn once in `ui::art`: the running window gets it from `rune::set_app_icon`, the executable from `build.rs` embedding `assets/app.ico`. Without the embedded copy the shell falls back to a generic icon wherever it reads the file rather than the window |
| Windows process | GUI subsystem in every profile. A console-subsystem build made Windows open a terminal window on launch, and that terminal's icon was the one in the taskbar; the startup banner reattaches to a parent console so shell runs still print |
| Instrument picker | A wrapping grid, not a horizontally scrolling filmstrip. The column count comes from the measured width of `channels-body` (`Cx::layout_with`, which re-runs the view only when that width moves and settles on the first layout), and rows are chunked by hand because Rune has no flex-wrap |
| Tile art | Thumbnails are square (the card width minus its 8 pt padding), so a full-disc reference image is never cropped to a band. Both type lines — name and wavelength — are single-line and cut to the tile's inner width, so every card is exactly one size |
| Data migration from Solar Go | Not required |
| Interactive solar-system scene | Out of scope for v1; the observatory is the product |

## Verification log

| Date | Command | Result |
|---|---|---|
| 2026-11-14 | `cargo check --locked` | pass, no warnings |
| 2026-11-14 | `cargo clippy --all-targets --locked -- -D warnings` | clean |
| 2026-11-14 | `cargo test --locked` | 16 passed, 1 ignored |
| 2026-11-14 | `--scene demo --snap` at 1180×820 @2× | titlebar, caption insets, aurora, hero, filmstrip render correctly |
| 2026-11-14 | `--scene gallery/detail/settings/statistics/about/crop/spanish` | all render; no clipping or overlap |
| 2026-11-15 | `cargo clippy --all-targets -- -D warnings` | clean |
| 2026-11-15 | `cargo test` | 19 passed, 1 ignored |
| 2026-11-15 | Channels hero at 1180×820 and 1000×760, EN and ES | three tiers and both temperature scales visible; filmstrip scrolls at the minimum size instead of clipping |
| 2026-11-17 | `cargo check --all-targets` | pass, no warnings |
| 2026-11-17 | `cargo clippy --all-targets -- -D warnings` | clean |
| 2026-11-18 | `cargo test --locked --offline` | 40 passed, 1 ignored |
| 2026-11-17 | macOS backend: `cargo clippy --target x86_64-apple-darwin -- -D warnings` over a scratch crate holding `wallpaper/*` verbatim | clean, against the real `objc2` 0.6.5 / `objc2-app-kit` 0.3.2 APIs; found and fixed a missing `NSEnumerator` feature before shipping |
| 2026-11-17 | Cross-compiling the whole crate from Windows to Linux/macOS | blocked by the host having no C toolchain for the target (`ring`, `cc`); left to the new CI matrix |
| 2026-11-17 | `--scene channels --snap` at 1180×820 @2× after the split | identical shell, 257 elements, no tree warnings |
| 2026-11-17 | `Cargo.lock`: who depends on which `objc2` | both versions predate the wallpaper backend — `winit`/`accesskit_macos` pull 0.5.3, `wgpu-hal`/`fontique`/`arboard`/rune pull 0.6.5 |
| 2026-11-17 | `GetIconInfo` + `GetDIBits` on the running window's `HICON`s | 256 and 32 px both hold the Sun: alpha 255 at the centre, 0 at the corners. The "hollow disc" seen earlier was a `DrawIconEx` dump artifact, not the icon |
| 2026-11-17 | `ExtractIconEx` on the built `.exe` | 1 icon resource; its 32 and 16 px bitmaps decode to the Sun with the same alpha |
| 2026-11-17 | Launching `target\debug\solar-rust.exe` | before: Windows Terminal opened with the exe path as its title, so the taskbar showed the terminal's icon. After `windows_subsystem`: no terminal, `MainWindowTitle` is `Solar Observatory`, banner still on a redirected stderr |
| 2026-11-17 | `cargo clippy --all-targets --locked -- -D warnings` | clean |
| 2026-11-18 | `cargo clippy --all-targets --locked --offline -- -D warnings` | clean |
| 2026-11-17 | `--scene channels --snap` at 1180×820, 1000×760 and 1600×1000 @2×, EN and ES | 5, 4 and 8 columns with no card past the body's right edge (`channels-body` 896 pt wide, last card ends at 1155 of 1156 at 1180); 19 tiles wrap to 4, 5 and 3 rows and the short row stays left-aligned |
| 2026-11-17 | `ui::views::channels::tests::instrument_grid_wraps_within_the_body_width` | every card inside the body rect, at least two cards on the first row, and at least one on a lower row |
| 2026-11-17 | `ui::views::channels::tests::a_filtered_grid_keeps_the_same_card_width` | the HMI filter keeps exactly its channels, one card width, every row starting at the grid's left edge and wrapping to a second row |
| 2026-11-17 | 19 tiles at 1600×1400, measured from `--tree` | every card is 155×195.25, and no text node reaches past its card's right edge — the composite wavelengths ("211 / 193 / 171 ANG…") are cut instead of painting over the neighbour |

Two things this log cannot say: no Linux desktop session and no Mac have run the new adapters.
Both are covered by unit tests or by the typecheck above, and by CI from now on, but a green
build is not the same as a wallpaper that changed.

---

*Update this file when a gap changes state. Keep the Snapshot table truthful.*

//! The Mission Deck shell: aurora ground, glass chrome, and the page frame.

mod about;
mod channels;
mod chrome;
mod crop;
mod detail;
mod gallery;
mod settings;
mod statistics;

use rune::{Color, Cx, El, Pixels, Shape, SurfaceFit, canvas, col, el, row};
use rune_kit::Kit;
use solar_rust::domain::Mode;

use super::{Page, State, icons, theme};
use theme::Tokens;

/// Everything a page needs to paint. Tokens are `'static`, so this stays cheap to copy.
#[derive(Clone, Copy)]
pub struct Paint {
    pub t: &'static Tokens,
    pub k: &'static Kit,
}

pub fn paint() -> Paint {
    Paint {
        t: Tokens::get(),
        k: theme::kit(),
    }
}

pub fn color_for_mode(mode: Mode) -> &'static str {
    match mode {
        Mode::Fit => "Fit",
        Mode::Crop => "Crop",
        Mode::Center => "Center",
        Mode::Span => "Span",
        Mode::SolarCrop => "Solar crop",
    }
}

/// One string, two languages. Instrument names, units and file paths stay as they are.
pub fn tr(state: &State, en: &'static str, es: &'static str) -> &'static str {
    match state.archive.preferences.language {
        solar_rust::domain::Language::En => en,
        solar_rust::domain::Language::Es => es,
    }
}

/// Plasma temperature in the unit that reads best: millions of kelvin when the channel lives
/// in the corona, plain kelvin when it watches the cooler layers, collapsed to one value.
pub fn temperature(channel: &solar_rust::domain::catalog::Channel) -> String {
    let low = channel.kelvin_min;
    let high = channel.kelvin_max;
    let single = (high - low).abs() < 50.0;
    if low >= 1_000_000.0 {
        if single {
            format!("{} MK", millions(low))
        } else {
            format!("{}–{} MK", millions(low), millions(high))
        }
    } else if single {
        format!("{} K", grouped(low, ' '))
    } else {
        format!("{}–{} K", grouped(low, ' '), grouped(high, ' '))
    }
}

/// The same temperature in the reader's everyday scale: Celsius in Spanish, Fahrenheit in
/// English. Millions collapse to `20M`; thousands are grouped so they read as a number.
pub fn temperature_equivalent(
    channel: &solar_rust::domain::catalog::Channel,
    language: solar_rust::domain::Language,
) -> String {
    let spanish = language == solar_rust::domain::Language::Es;
    let scale = |kelvin: f64| {
        let celsius = kelvin - 273.15;
        if spanish {
            celsius
        } else {
            celsius * 1.8 + 32.0
        }
    };
    let unit = if spanish { "°C" } else { "°F" };
    let separator = if spanish { ' ' } else { ',' };
    let low = scale(channel.kelvin_min);
    let high = scale(channel.kelvin_max);
    if (high - low).abs() < 1.0 {
        format!("≈ {} {unit}", degrees(low, separator))
    } else {
        format!(
            "≈ {}–{} {unit}",
            degrees(low, separator),
            degrees(high, separator)
        )
    }
}

fn degrees(value: f64, separator: char) -> String {
    if value.abs() >= 1_000_000.0 {
        format!("{}M", millions(value))
    } else {
        grouped(value, separator)
    }
}

/// A million-scale number as the reader counts it: `6.3`, `20`, or `1.3`.
fn millions(value: f64) -> String {
    let millions = value / 1_000_000.0;
    if (millions - millions.round()).abs() < 0.05 {
        format!("{millions:.0}")
    } else {
        format!("{millions:.1}")
    }
}

fn grouped(value: f64, separator: char) -> String {
    // `≈` already says this is not exact, so a number this large rounds to the nearest hundred
    // instead of claiming precision the reader cannot use.
    let rounded = if value.abs() >= 10_000.0 {
        (value / 100.0).round() as i64 * 100
    } else {
        value.round() as i64
    };
    let digits = rounded.abs().to_string();
    let mut out = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(separator);
        }
        out.push(digit);
    }
    if rounded < 0 {
        out.insert(0, '−');
    }
    out
}

/// A picture surface: sunken ground, rounded clip, and the image when one is loaded.
pub fn picture(px: Option<&Pixels>, fit: SurfaceFit, radius: f32) -> El {
    let t = Tokens::get();
    let mut node = el()
        .w_full()
        .h_full()
        .fill(t.panel_sunken)
        .radius(radius)
        .clip_rounded(radius);
    match px {
        Some(px) => node = node.pixels_fit(px, fit),
        None => {
            node = node
                .center()
                .child(rune::icon(&icons::SUN, 24.0).color(t.text_4))
        }
    }
    node.a11y_hidden()
}

/// A page header: eyebrow, title, one-line description, and trailing actions.
pub fn page_header(p: Paint, eyebrow: &str, title: &str, detail: &str, actions: Vec<El>) -> El {
    let copy = col()
        .gap(6.0)
        .grow(1.0)
        .min_w(0.0)
        .child(super::components::micro(eyebrow, p.t.text_3))
        .child(super::components::label(title, theme::H1, p.t.text))
        .child(super::components::label(detail, theme::BODY, p.t.text_2).wrap());
    let mut actions_row = row().gap(10.0).center_y().shrink();
    for action in actions {
        actions_row = actions_row.child(action);
    }
    row()
        .key("page-header")
        .w_full()
        .gap(20.0)
        .center_y()
        .justify_between()
        .child(copy)
        .child(actions_row)
}

pub fn view(state: &State, cx: &mut Cx<State>) -> El {
    let p = paint();
    // Windows draws its caption buttons over our band; the insets say how much they take so
    // the app's own controls stop short of them instead of sitting underneath.
    let insets = cx.chrome_insets();
    cx.caption_color(p.t.text_3);
    let body = match state.page {
        Page::Channels => channels::view(state, p, cx),
        Page::Gallery => gallery::view(state, p),
        Page::Detail => detail::view(state, p),
        Page::Settings => settings::view(state, p),
        Page::Statistics => statistics::view(state, p),
        Page::About => about::view(state, p),
    }
    // Keep navigation immediate and legible: a short, fade-free nudge avoids a black gap
    // between fully transparent page subtrees while preserving a little directional motion.
    .transition(rune::Transition::offset(0.0, 8.0));
    let base = el()
        .key("root")
        .w_full()
        .h_full()
        .fill(p.t.bg_deep)
        .child(background(p))
        .child(
            col()
                .key("shell")
                .w_full()
                .h_full()
                .child(chrome::titlebar(state, p, insets))
                .child(notice(state, p))
                .child(
                    row()
                        .key("workspace")
                        .grow(1.0)
                        .w_full()
                        .min_h(0.0)
                        .child(chrome::sidebar(state, p))
                        .child(
                            col()
                                .key("page")
                                .grow(1.0)
                                .min_w(0.0)
                                .min_h(0.0)
                                .h_full()
                                .padding(24.0)
                                .child(body),
                        ),
                ),
        );
    if state.draft.is_some() {
        let size = cx.layout_size("crop-frame");
        rune_kit::root(base, [crop::overlay(state, p, size)])
    } else {
        rune_kit::root(base, [])
    }
}

/// A shell-level notice: the last failure, kept in the flow instead of a modal.
fn notice(state: &State, p: Paint) -> El {
    let Some(error) = &state.error else {
        return el().h(0.0).w_full().a11y_hidden();
    };
    let t = p.t;
    row()
        .key("notice")
        .w_full()
        .h(34.0)
        .px(20.0)
        .gap(9.0)
        .center_y()
        .fill(t.danger.alpha(0.10))
        .border_sides([0.0, 0.0, 1.0, 0.0], t.danger.alpha(0.28))
        .child(rune::icon(&super::icons::WARN, 14.0).color(t.danger))
        .child(
            super::components::label(error.clone(), theme::MONO_SM, t.text_2)
                .truncate()
                .grow(1.0),
        )
}

/// The aurora ground: three low-alpha colour fields over near-black, a technical grid, and a
/// vignette that keeps the eye on the content. No external image, no window transparency.
fn background(p: Paint) -> El {
    let t = p.t;
    el().key("backdrop")
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .fill(t.bg_deep)
        .child(
            el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .radial_gradient(
                    [0.16, 0.06],
                    [720.0, 560.0],
                    [
                        (0.0, t.teal.alpha(0.15)),
                        (0.38, t.teal.alpha(0.055)),
                        (0.72, t.teal.alpha(0.015)),
                        (1.0, Color::TRANSPARENT),
                    ],
                )
                .a11y_hidden(),
        )
        .child(
            el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .radial_gradient(
                    [0.92, 0.96],
                    [780.0, 640.0],
                    [
                        (0.0, t.violet.alpha(0.13)),
                        (0.40, t.violet.alpha(0.045)),
                        (0.75, t.violet.alpha(0.012)),
                        (1.0, Color::TRANSPARENT),
                    ],
                )
                .a11y_hidden(),
        )
        .child(
            el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .radial_gradient(
                    [0.62, 0.30],
                    [520.0, 420.0],
                    [
                        (0.0, t.amber.alpha(0.055)),
                        (0.45, t.amber.alpha(0.018)),
                        (1.0, Color::TRANSPARENT),
                    ],
                )
                .a11y_hidden(),
        )
        .child(grid_overlay(p))
        .child(breathing_bloom(p))
        .child(
            el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .linear_gradient(
                    std::f32::consts::PI,
                    [
                        (0.0, Color::BLACK.alpha(0.26)),
                        (0.20, Color::TRANSPARENT),
                        (0.72, Color::TRANSPARENT),
                        (1.0, Color::BLACK.alpha(0.50)),
                    ],
                )
                .a11y_hidden(),
        )
}

/// One wide field over the static aurora, breathing on a long clock. It adds light rather than
/// dimming the ground, so the shell never looks dead between syncs.
fn breathing_bloom(p: Paint) -> El {
    let t = p.t;
    el().key("bloom")
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .radial_gradient(
            [0.46, 0.40],
            [880.0, 720.0],
            [
                (0.0, t.teal.alpha(0.085)),
                (0.34, t.violet.alpha(0.045)),
                (0.70, t.teal.alpha(0.012)),
                (1.0, Color::TRANSPARENT),
            ],
        )
        .looping(rune::LoopSpec::opacity(23.0, 0.0, 0.38, 1.0))
        .a11y_hidden()
}

fn grid_overlay(_p: Paint) -> El {
    let ink = Color::WHITE.alpha(0.016);
    let mut shape = Shape::new().view_box(100.0, 100.0);
    let mut i = 0;
    while i <= 100 {
        let v = i as f32;
        shape = shape.line([v, 0.0], [v, 100.0], 0.5, rune::shape::LineCap::Butt, ink);
        shape = shape.line([0.0, v], [100.0, v], 0.5, rune::shape::LineCap::Butt, ink);
        i += 5;
    }
    canvas(shape)
        .key("grid")
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .a11y_hidden()
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i + 1 < UNITS.len() {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{v:.0} {}", UNITS[i])
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}

/// Unix seconds to `HH:MM UTC` without pulling in a date library.
pub fn utc_hhmm(seconds: u64) -> String {
    let secs = seconds % 86_400;
    format!("{:02}:{:02} UTC", secs / 3_600, (secs % 3_600) / 60)
}

/// Unix seconds to `YYYY-MM-DD` using the civil-from-days algorithm.
pub fn utc_date(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64 + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use solar_rust::domain::{Language, catalog};

    #[test]
    fn temperature_picks_the_unit_that_reads_best() {
        let chromosphere = catalog::channel("0304").expect("0304");
        assert_eq!(temperature(chromosphere), "50 000 K");
        let corona = catalog::channel("0131").expect("0131");
        assert_eq!(temperature(corona), "10–16 MK");
        let photosphere = catalog::channel("HMII").expect("HMII");
        assert_eq!(temperature(photosphere), "5 000–6 000 K");
    }

    #[test]
    fn temperature_equivalent_speaks_the_readers_scale() {
        let channel = catalog::channel("0304").expect("0304");
        assert_eq!(temperature_equivalent(channel, Language::Es), "≈ 49 700 °C");
        assert_eq!(temperature_equivalent(channel, Language::En), "≈ 89,500 °F");
    }

    #[test]
    fn temperature_equivalent_keeps_a_range_and_collapses_millions() {
        let channel = catalog::channel("0131").expect("0131");
        assert_eq!(
            temperature_equivalent(channel, Language::Es),
            "≈ 10M–16M °C"
        );
    }
}

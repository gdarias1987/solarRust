//! Small Mission Deck primitives shared by every page. Everything here is built from `El`
//! so the shell keeps one visual language; `rune-kit` supplies the controls.

use rune::{Color, El, Icon, TextStyle, col, el, icon, row, spacer, text};
use solar_rust::domain::Mode;

use super::{icons, theme};
use theme::Tokens;

pub fn label(value: impl Into<String>, font: TextStyle, color: Color) -> El {
    text(value).font(font).color(color)
}

/// A tracked, upper-case instrument label.
pub fn micro(value: impl Into<String>, color: Color) -> El {
    text(value.into().to_uppercase())
        .font(theme::MICRO)
        .color(color)
}

/// The Sun mark: a warm core inside a soft halo, the brand's only ornament.
pub fn sun_mark(size: f32) -> El {
    let t = Tokens::get();
    el().size(size, size)
        .radius(size / 2.0)
        .fill(t.amber.alpha(0.10))
        .radial_gradient(
            [0.5, 0.46],
            size * 0.62,
            [
                (0.0, t.amber),
                (0.32, t.amber.alpha(0.92)),
                (0.62, t.amber_dim.alpha(0.38)),
                (1.0, Color::TRANSPARENT),
            ],
        )
        .border(1.0, t.amber.alpha(0.35))
        .center()
        .child(icon(&icons::SUN, size * 0.56).color(Color::WHITE.alpha(0.94)))
        .a11y_hidden()
}

pub fn brand() -> El {
    let t = Tokens::get();
    row()
        .gap(10.0)
        .center_y()
        .child(sun_mark(30.0).looping(rune::LoopSpec::opacity(5.6, 0.0, 0.84, 1.0)))
        .child(
            col()
                .gap(1.0)
                .child(label("SOLAR", theme::WORDMARK, t.text))
                .child(micro("Observatory", t.text_3)),
        )
}

pub fn chip(value: impl Into<String>, color: Color) -> El {
    el().h(22.0)
        .px(9.0)
        .center_y()
        .radius(6.0)
        .fill(color.alpha(0.10))
        .border(1.0, color.alpha(0.26))
        .child(label(value, theme::MONO_SM, color))
        .a11y_hidden()
}

pub fn hairline() -> El {
    let t = Tokens::get();
    el().h(1.0).w_full().fill(t.hairline_soft).a11y_hidden()
}

pub fn section_label(value: impl Into<String>) -> El {
    micro(value, Tokens::get().text_3)
}

/// A labelled row: small muted key, right-aligned value.
pub fn meta_row(key: impl Into<String>, value: impl Into<String>) -> El {
    let t = Tokens::get();
    row()
        .w_full()
        .h(26.0)
        .center_y()
        .justify_between()
        .child(label(key, theme::BODY_SM, t.text_3))
        .child(label(value, theme::MONO, t.text_2).align(rune::Align::End))
}

/// A metadata pair whose value needs its own line: long prose never collides with the key.
pub fn stack_row(key: impl Into<String>, value: impl Into<String>) -> El {
    let t = Tokens::get();
    col()
        .w_full()
        .gap(2.0)
        .child(micro(key, t.text_3))
        .child(label(value, theme::BODY_SM, t.text_2).wrap())
}

pub fn nav_row<S: 'static>(
    key: &'static str,
    glyph: &'static Icon,
    title: &'static str,
    selected: bool,
    on_press: impl Fn(&mut S) + 'static,
) -> El {
    let t = Tokens::get();
    let ink = if selected { t.text } else { t.text_2 };
    let mut nav = row()
        .key(key)
        .w_full()
        .h(38.0)
        .px(10.0)
        .gap(10.0)
        .center_y()
        .radius(9.0)
        .hover(t.glass_soft)
        .press(t.glass)
        .motion(rune::Spring::new(0.26, 0.0))
        .focusable(true)
        .label(title)
        .cursor(rune::Cursor::Pointer)
        .on_press(on_press)
        .child(icon(glyph, 16.0).color(if selected { t.teal } else { t.text_3 }))
        .child(label(title, theme::BODY, ink));
    if selected {
        nav = nav
            .fill(t.teal.alpha(0.10))
            .border(1.0, t.teal.alpha(0.22))
            .child(
                el().key("nav-dot")
                    .absolute(-1.0, 11.0)
                    .size(3.0, 16.0)
                    .radius(2.0)
                    .fill(t.teal)
                    .a11y_hidden(),
            );
    }
    nav
}

pub fn icon_button<S: 'static>(
    glyph: &'static Icon,
    tooltip: &'static str,
    on_press: impl Fn(&mut S) + 'static,
) -> El {
    let t = Tokens::get();
    el().size(32.0, 32.0)
        .center()
        .radius(9.0)
        .fill(t.glass_soft)
        .border(1.0, t.hairline)
        .hover(t.hairline_soft)
        .press(t.glass)
        .motion(rune::Spring::new(0.24, 0.0))
        .focusable(true)
        .label(tooltip)
        .help(tooltip)
        .cursor(rune::Cursor::Pointer)
        .on_press(on_press)
        .child(icon(glyph, 16.0).color(t.text_2))
}

/// A status light. `glow` adds the soft halo used for live data.
pub fn status_dot(color: Color, glow: bool) -> El {
    let mut dot = el().size(7.0, 7.0).radius(4.0).fill(color);
    if glow {
        dot = dot
            .shadow(0.0, 8.0, color.alpha(0.55))
            .shadow(0.0, 2.0, color.alpha(0.8));
    }
    el().size(12.0, 12.0).center().child(dot).a11y_hidden()
}

/// A status light that breathes: quick while work is in flight, slow at rest, dark when the
/// shell has nothing to watch.
pub fn pulse_dot(color: Color, period: f64, glow: bool) -> El {
    let mut dot = el().size(7.0, 7.0).radius(4.0).fill(color);
    if glow {
        dot = dot
            .shadow(0.0, 8.0, color.alpha(0.55))
            .shadow(0.0, 2.0, color.alpha(0.8));
    }
    el().size(12.0, 12.0)
        .center()
        .child(dot.looping(rune::LoopSpec::opacity(
            period,
            0.0,
            if glow { 0.58 } else { 0.30 },
            1.0,
        )))
        .a11y_hidden()
}

pub fn stat_tile(caption: &str, value: String, unit: &str, accent: Color) -> El {
    let t = Tokens::get();
    col()
        .key(format!("stat-{caption}"))
        .grow(1.0)
        .padding(16.0)
        .gap(6.0)
        .radius(14.0)
        .fill(t.panel)
        .border(1.0, t.hairline)
        .shadow(10.0, 28.0, Color::BLACK.alpha(0.22))
        .child(micro(caption, t.text_3))
        .child(
            row()
                .center_y()
                .gap(6.0)
                .child(label(value, theme::NUM, t.text))
                .child(label(unit, theme::MONO, t.text_3).mt(3.0)),
        )
        .child(
            el().h(2.0)
                .w(34.0)
                .radius(1.0)
                .fill(accent)
                .shadow(0.0, 6.0, accent.alpha(0.55)),
        )
}

/// A tiny proportional bar for the statistics page.
pub fn bar(fraction: f32, accent: Color) -> El {
    let t = Tokens::get();
    el().key(format!("bar-{:.3}", fraction))
        .h(6.0)
        .w_full()
        .radius(3.0)
        .fill(t.panel_sunken)
        .child(
            el().h(6.0)
                .w_frac(fraction.clamp(0.0, 1.0))
                .radius(3.0)
                .fill(accent)
                .a11y_hidden(),
        )
        .a11y_hidden()
}

/// Mini diagram of how the wallpaper fits the screen. Drawn with boxes, no image needed.
pub fn mode_diagram(mode: Mode, active: bool) -> El {
    let t = Tokens::get();
    let (w, h) = (64.0, 40.0);
    let (ink, dim) = if active {
        (t.teal, t.teal.alpha(0.16))
    } else {
        (t.text_3, t.hairline_soft)
    };
    let mut frame = el()
        .size(w, h)
        .radius(6.0)
        .fill(t.panel_sunken)
        .border(1.0, if active { ink.alpha(0.55) } else { t.hairline })
        .a11y_hidden();
    let inner = match mode {
        Mode::Fit => (34.0, 19.0),
        Mode::Crop => (52.0, 30.0),
        Mode::Center => (26.0, 26.0),
        Mode::Span => (60.0, 37.0),
        Mode::SolarCrop => (26.0, 26.0),
    };
    frame = frame.child(
        el().absolute((w - inner.0) / 2.0, (h - inner.1) / 2.0)
            .size(inner.0, inner.1)
            .radius(if mode == Mode::SolarCrop { 13.0 } else { 2.5 })
            .fill(dim)
            .border(1.0, ink.alpha(if active { 0.8 } else { 0.4 }))
            .a11y_hidden(),
    );
    if mode == Mode::Crop {
        frame = frame.child(
            el().absolute((w - inner.0) / 2.0, (h - inner.1) / 2.0)
                .size(inner.0, inner.1)
                .border_gradient(1.0, t.amber.alpha(0.7), t.amber.alpha(0.08))
                .a11y_hidden(),
        );
    }
    frame
}

/// The hero's primary action.
pub fn accent_button<S: 'static>(
    label_text: &str,
    glyph: &'static Icon,
    on_press: impl Fn(&mut S) + 'static,
) -> El {
    let t = Tokens::get();
    row()
        .key(format!("accent-{label_text}"))
        .h(34.0)
        .px(14.0)
        .gap(8.0)
        .center_y()
        .radius(10.0)
        .fill(t.teal)
        .shadow(4.0, 16.0, t.teal.alpha(0.35))
        .shadow(0.0, 1.0, Color::BLACK.alpha(0.35))
        .hover(Color::WHITE.alpha(0.10))
        .press(Color::BLACK.alpha(0.12))
        .motion(rune::Spring::new(0.24, 0.0))
        .focusable(true)
        .label(label_text)
        .cursor(rune::Cursor::Pointer)
        .on_press(on_press)
        .child(icon(glyph, 15.0).color(t.bg_deep))
        .child(label(label_text, theme::H3, t.bg_deep))
}

/// A lower-emphasis action.
pub fn ghost_button<S: 'static>(
    label_text: &str,
    glyph: &'static Icon,
    on_press: impl Fn(&mut S) + 'static,
) -> El {
    let t = Tokens::get();
    row()
        .key(format!("ghost-{label_text}"))
        .h(34.0)
        .px(13.0)
        .gap(8.0)
        .center_y()
        .radius(10.0)
        .fill(t.glass_soft)
        .border(1.0, t.hairline)
        .hover(t.glass)
        .press(t.glass_strong)
        .motion(rune::Spring::new(0.24, 0.0))
        .focusable(true)
        .label(label_text)
        .cursor(rune::Cursor::Pointer)
        .on_press(on_press)
        .child(icon(glyph, 15.0).color(t.text_2))
        .child(label(label_text, theme::H3, t.text))
}

/// Spaces the toolbar's left and right groups.
pub fn spring() -> El {
    spacer().grow(1.0)
}

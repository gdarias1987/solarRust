//! The window chrome: a glass titlebar with live sync state, and the navigation rail.

use rune::{ChromeInsets, El, col, el, icon, row, spacer};
use rune_kit::SegmentedControl;
use solar_rust::{application::Preference, domain::Language};

use crate::ui::{
    Page, State,
    components::{self},
    icons, theme,
};

use super::Paint;

pub fn titlebar(state: &State, p: Paint, insets: ChromeInsets) -> El {
    let t = p.t;
    let (label, ink) = if state.demo_mode {
        (super::tr(state, "Fixture", "Demo"), t.amber)
    } else if state.apply_feedback == Some(crate::ui::ApplyFeedback::Applying) {
        (super::tr(state, "Applying", "Aplicando"), t.teal)
    } else if state.busy {
        (super::tr(state, "Syncing", "Sincronizando"), t.amber)
    } else if state.error.is_some() {
        (super::tr(state, "Attention", "Atención"), t.danger)
    } else {
        (super::tr(state, "Synced", "Sincronizado"), t.teal)
    };
    let live = !state.busy && !state.demo_mode;
    let last = state.archive.records.iter().map(|r| r.downloaded_at).max();
    let stamp = last
        .map(super::utc_hhmm)
        .unwrap_or_else(|| super::tr(state, "no capture", "sin captura").into());

    let status = row()
        .gap(8.0)
        .center_y()
        .child(components::pulse_dot(
            ink,
            if state.busy || state.apply_feedback == Some(crate::ui::ApplyFeedback::Applying) {
                1.1
            } else {
                3.4
            },
            live,
        ))
        .child(components::label(label, theme::MONO_SM, ink))
        .child(el().w(1.0).h(12.0).fill(t.hairline).a11y_hidden())
        .child(components::label(stamp, theme::MONO_SM, t.text_3));

    row()
        .key("titlebar")
        .w_full()
        .h(crate::ui::TITLEBAR_HEIGHT)
        .pl(16.0 + insets.leading)
        .pr(10.0 + insets.trailing)
        .gap(14.0)
        .center_y()
        .backdrop_blur(16.0)
        .fill(t.glass)
        .border_sides([0.0, 0.0, 1.0, 0.0], t.hairline)
        .window_drag()
        // A single sheen along the top edge: glass catching the room's light.
        .child(
            el().pin(Some(0.0), Some(0.0), Some(0.0), None)
                .h(26.0)
                .linear_gradient(
                    std::f32::consts::PI,
                    [
                        (0.0, rune::Color::WHITE.alpha(0.035)),
                        (1.0, rune::Color::TRANSPARENT),
                    ],
                )
                .a11y_hidden(),
        )
        .child(components::brand())
        .child(el().w(1.0).h(20.0).fill(t.hairline).a11y_hidden())
        .child(components::micro(
            super::tr(state, "Mission deck", "Cubierta de misión"),
            t.text_4,
        ))
        .child(components::spring())
        .child(status)
        .child(components::spring())
        .child(
            row()
                .gap(8.0)
                .center_y()
                .child(components::icon_button(
                    &icons::SYNC,
                    "Check for a new capture",
                    |s: &mut State| s.acquire(true, false),
                ))
                .child(components::icon_button(
                    &icons::SLIDERS,
                    "Settings",
                    |s: &mut State| s.go(Page::Settings),
                )),
        )
}

pub fn sidebar(state: &State, p: Paint) -> El {
    let t = p.t;
    let records = state.archive.records.len();
    let retention = state.archive.preferences.retention.max(1);
    let bytes: u64 = state.archive.records.iter().map(|r| r.bytes).sum();
    let fraction = records as f32 / retention as f32;

    let storage = col()
        .key("storage-card")
        .w_full()
        .padding(12.0)
        .gap(8.0)
        .radius(12.0)
        .fill(t.glass_soft)
        .border(1.0, t.hairline)
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .child(components::micro(
                    super::tr(state, "Archive", "Archivo"),
                    t.text_3,
                ))
                .child(components::label(
                    format!("{records}/{retention}"),
                    theme::MONO,
                    t.text_2,
                )),
        )
        .child(components::bar(
            fraction.min(1.0),
            if fraction >= 0.9 { t.amber } else { t.teal },
        ))
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .child(components::label(
                    super::format_bytes(bytes),
                    theme::MONO_SM,
                    t.text_3,
                ))
                .child(components::label(
                    super::tr(state, "kept locally", "guardado local"),
                    theme::MONO_SM,
                    t.text_4,
                )),
        );

    let language = SegmentedControl::new(
        &["EN", "ES"],
        Some(if state.archive.preferences.language == Language::Es {
            1
        } else {
            0
        }),
    )
    .fit()
    .label(super::tr(state, "Language", "Idioma"))
    .on_pick(|s: &mut State, index, _| {
        s.preference(Preference::Language(if index == 1 {
            Language::Es
        } else {
            Language::En
        }))
    })
    .build(p.k);

    col()
        .key("sidebar")
        .w(236.0)
        .h_full()
        .pt(16.0)
        .pb(14.0)
        .px(12.0)
        .gap(3.0)
        .backdrop_blur(16.0)
        .fill(t.glass)
        .border_sides([0.0, 0.0, 0.0, 1.0], t.hairline)
        .child(
            col()
                .key("nav-operations")
                .w_full()
                .gap(3.0)
                .child(
                    components::section_label(super::tr(state, "Operations", "Operaciones"))
                        .pl(10.0)
                        .pb(6.0)
                        .pt(2.0),
                )
                .child(components::nav_row(
                    "nav-channels",
                    &icons::LAYERS,
                    super::tr(state, "Channels", "Canales"),
                    state.page == Page::Channels,
                    |s: &mut State| s.go(Page::Channels),
                ))
                .child(components::nav_row(
                    "nav-gallery",
                    &icons::GALLERY,
                    super::tr(state, "Archive", "Archivo"),
                    state.page == Page::Gallery || state.page == Page::Detail,
                    |s: &mut State| s.go(Page::Gallery),
                )),
        )
        .child(
            col()
                .key("nav-system")
                .w_full()
                .gap(3.0)
                .pt(16.0)
                .child(
                    components::section_label(super::tr(state, "System", "Sistema"))
                        .pl(10.0)
                        .pb(6.0),
                )
                .child(components::nav_row(
                    "nav-settings",
                    &icons::SLIDERS,
                    super::tr(state, "Settings", "Ajustes"),
                    state.page == Page::Settings,
                    |s: &mut State| s.go(Page::Settings),
                ))
                .child(components::nav_row(
                    "nav-statistics",
                    &icons::CHART,
                    super::tr(state, "Statistics", "Estadísticas"),
                    state.page == Page::Statistics,
                    |s: &mut State| s.go(Page::Statistics),
                ))
                .child(components::nav_row(
                    "nav-about",
                    &icons::INFO,
                    super::tr(state, "About", "Acerca de"),
                    state.page == Page::About,
                    |s: &mut State| s.go(Page::About),
                )),
        )
        .child(spacer().grow(1.0))
        .child(
            col()
                .key("nav-footer")
                .w_full()
                .gap(10.0)
                .child(storage)
                .child(language),
        )
}

/// A compact breadcrumb used by page headers in place of a tab strip.
pub fn breadcrumb(parts: &[&str]) -> El {
    let t = crate::ui::theme::Tokens::get();
    let mut line = row().gap(7.0).center_y();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            line = line.child(icon(&icons::CHEVRON_RIGHT, 11.0).color(t.text_4));
        }
        line = line.child(components::label(*part, theme::MONO_SM, t.text_3));
    }
    line
}

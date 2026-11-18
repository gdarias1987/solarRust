//! Settings: how the observatory looks, how often it checks, and what it keeps.

use rune::{Cursor, El, ScrollSpec, col, el, row};
use rune_kit::SegmentedControl;
use solar_rust::{
    application::Preference,
    domain::{Language, Mode},
};

use crate::ui::{
    State,
    components::{self, accent_button, ghost_button, meta_row, mode_diagram},
    icons, theme,
};

use super::{Paint, format_bytes, page_header, tr};

const INTERVALS: [(u64, &str); 5] = [
    (60, "1 min"),
    (300, "5 min"),
    (900, "15 min"),
    (1_800, "30 min"),
    (3_600, "1 h"),
];
const RETENTIONS: [usize; 4] = [5, 10, 25, 50];

pub fn view(state: &State, p: Paint) -> El {
    let header = page_header(
        p,
        tr(state, "System / settings", "Sistema / ajustes"),
        tr(state, "Settings", "Ajustes"),
        tr(
            state,
            "Appearance, cadence and storage. Everything here is stored with the archive.",
            "Apariencia, cadencia y almacenamiento. Todo esto se guarda con el archivo.",
        ),
        vec![],
    );

    let body = col()
        .key("settings-body")
        .w_full()
        .gap(14.0)
        .child(appearance(state, p))
        .child(cadence(state, p))
        .child(language(state, p))
        .child(storage(state, p))
        .child(crop(state, p));

    col()
        .key("settings-page")
        .w_full()
        .h_full()
        .min_h(0.0)
        .gap(16.0)
        .child(header)
        .child(
            col()
                .key("settings-scroll")
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(body),
        )
}

fn card(title: &str, subtitle: &str, p: Paint) -> El {
    let t = p.t;
    col()
        .key(format!("card-{title}"))
        .w_full()
        .padding(16.0)
        .gap(12.0)
        .radius(16.0)
        .fill(t.panel)
        .border(1.0, t.hairline)
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .child(components::section_label(title))
                .child(components::micro(subtitle, t.text_4)),
        )
}

fn appearance(state: &State, p: Paint) -> El {
    let t = p.t;
    let persisted = state.archive.preferences.mode;
    let active = state.appearance_mode();
    let mut tiles = row().w_full().gap(10.0);
    for mode in Mode::ALL {
        let cursor = Cursor::Pointer;
        let mut tile = col()
            .key(format!("mode-{:?}", mode))
            .grow(1.0)
            .h(104.0)
            .padding(10.0)
            .gap(8.0)
            .radius(12.0)
            .fill(if mode == active {
                t.panel_hi
            } else {
                t.glass_soft
            })
            .border(
                1.0,
                if mode == active {
                    t.teal.alpha(0.45)
                } else {
                    t.hairline
                },
            )
            .hover(t.panel_hi)
            .focusable(true)
            .checked(mode == active)
            .label(super::color_for_mode(mode))
            .cursor(cursor)
            .on_press(move |s: &mut State| s.select_appearance(mode))
            .child(
                el().w_full()
                    .h(44.0)
                    .center()
                    .child(mode_diagram(mode, mode == active)),
            )
            .child(components::label(
                super::color_for_mode(mode),
                theme::LABEL,
                if mode == active { t.text } else { t.text_2 },
            ));
        if mode == active {
            tile = tile.shadow(0.0, 12.0, t.teal.alpha(0.14));
        }
        tiles = tiles.child(tile);
    }
    let summary = match state.archive.preferences.language {
        Language::Es => format!("Fondo · {}", super::color_for_mode(active)),
        Language::En => format!("Wallpaper · {}", super::color_for_mode(active)),
    };
    let mut node = card(
        tr(state, "Appearance", "Apariencia"),
        &summary,
        p,
    )
    .child(tiles)
    .child(components::label(
        tr(
            state,
            "Fit keeps the full disc, crop fills the screen, span stretches edge to edge, and solar crop centres a 16:9 frame.",
            "Fit conserva el disco completo, crop llena la pantalla, span estira al borde y solar crop centra un encuadre 16:9.",
        ),
        theme::BODY_SM,
        t.text_3,
    )
    .wrap());
    if state.appearance_is_pending() {
        let status = components::micro(
            tr(state, "Saving changes...", "Guardando cambios..."),
            t.text_4,
        );
        node = if state.appearance_has_draft() {
            node.child(
                row()
                    .w_full()
                    .center_y()
                    .justify_end()
                    .gap(8.0)
                    .child(status)
                    .child(ghost_button(
                        tr(state, "Cancel", "Cancelar"),
                        &icons::CLOSE,
                        |s: &mut State| s.cancel_appearance(),
                    )),
            )
        } else {
            node.child(status)
        };
    } else if active != persisted {
        node = node.child(
            row()
                .w_full()
                .center_y()
                .justify_end()
                .gap(8.0)
                .child(ghost_button(
                    tr(state, "Cancel", "Cancelar"),
                    &icons::CLOSE,
                    |s: &mut State| s.cancel_appearance(),
                ))
                .child(accent_button(
                    tr(state, "Apply changes", "Aplicar cambios"),
                    &icons::APPLY,
                    |s: &mut State| s.apply_appearance(),
                )),
        );
    }
    node
}

fn cadence(state: &State, p: Paint) -> El {
    let t = p.t;
    let seconds = state.archive.preferences.interval_seconds;
    let index = INTERVALS
        .iter()
        .position(|(value, _)| *value == seconds)
        .or_else(|| INTERVALS.iter().position(|(value, _)| *value == 300))
        .unwrap_or(1);
    let retention = state.archive.preferences.retention;
    let retention_index = RETENTIONS
        .iter()
        .position(|value| *value == retention)
        .unwrap_or(1);

    card(
        tr(state, "Cadence", "Cadencia"),
        tr(state, "Automatic checks", "Chequeos automáticos"),
        p,
    )
    .child(
        row()
            .w_full()
            .center_y()
            .justify_between()
            .child(components::label(
                tr(state, "Check every", "Buscar cada"),
                theme::BODY,
                t.text_2,
            ))
            .child(
                SegmentedControl::new(
                    &INTERVALS
                        .iter()
                        .map(|(_, label)| *label)
                        .collect::<Vec<_>>(),
                    Some(index),
                )
                .fit()
                .label("Interval")
                .on_pick(|s: &mut State, i, _| {
                    if let Some((value, _)) = INTERVALS.get(i) {
                        s.preference(Preference::Interval(*value));
                    }
                })
                .build(p.k),
            ),
    )
    .child(components::hairline())
    .child(
        row()
            .w_full()
            .center_y()
            .justify_between()
            .child(components::label(
                tr(state, "Keep images", "Conservar imágenes"),
                theme::BODY,
                t.text_2,
            ))
            .child(
                SegmentedControl::new(
                    &RETENTIONS.iter().map(|v| v.to_string()).collect::<Vec<_>>(),
                    Some(retention_index),
                )
                .fit()
                .label("Retention")
                .on_pick(|s: &mut State, i, _| {
                    if let Some(value) = RETENTIONS.get(i) {
                        s.preference(Preference::Retention(*value));
                    }
                })
                .build(p.k),
            ),
    )
}

fn language(state: &State, p: Paint) -> El {
    let t = p.t;
    let spanish = state.archive.preferences.language == Language::Es;
    card(
        tr(state, "Language", "Idioma"),
        tr(state, "Interface", "Interfaz"),
        p,
    )
    .child(
        row()
            .w_full()
            .center_y()
            .justify_between()
            .child(
                col()
                    .gap(2.0)
                    .child(components::label(
                        tr(state, "Interface language", "Idioma de la interfaz"),
                        theme::BODY,
                        t.text_2,
                    ))
                    .child(components::micro(
                        tr(state, "Detected at first launch", "Detectado al inicio"),
                        t.text_4,
                    )),
            )
            .child(
                SegmentedControl::new(&["English", "Español"], Some(if spanish { 1 } else { 0 }))
                    .fit()
                    .label("Language")
                    .on_pick(|s: &mut State, i, _| {
                        s.preference(Preference::Language(if i == 1 {
                            Language::Es
                        } else {
                            Language::En
                        }))
                    })
                    .build(p.k),
            ),
    )
}

fn storage(state: &State, p: Paint) -> El {
    let t = p.t;
    let bytes: u64 = state.archive.records.iter().map(|r| r.bytes).sum();
    let path = solar_rust::infrastructure::storage::data_directory();
    card(
        tr(state, "Storage", "Almacenamiento"),
        tr(state, "Local only", "Solo local"),
        p,
    )
    .child(meta_row(
        tr(state, "Data directory", "Directorio de datos"),
        path.display().to_string(),
    ))
    .child(meta_row(
        tr(state, "Images", "Imágenes"),
        format!("{}", state.archive.records.len()),
    ))
    .child(meta_row(
        tr(state, "Disk usage", "Uso de disco"),
        format_bytes(bytes),
    ))
    .child(components::label(
        tr(
            state,
            "The archive is a single JSON index plus the original JPEGs, locked by this process.",
            "El archivo es un único índice JSON más los JPEG originales, bloqueado por este proceso.",
        ),
        theme::MONO_SM,
        t.text_4,
    )
    .wrap())
}

fn crop(state: &State, p: Paint) -> El {
    let t = p.t;
    let current = state.archive.preferences.crop.as_ref();
    let summary = match current {
        Some(c) => format!(
            "{} x {} at {:.0}, {:.0}",
            c.width.round(),
            c.height.round(),
            c.x.round(),
            c.y.round()
        ),
        None => tr(state, "Whole image", "Imagen completa").to_string(),
    };
    let mut node = card(
        tr(state, "Desktop crop", "Recorte de escritorio"),
        tr(state, "Original pixels", "Píxeles originales"),
        p,
    )
    .child(meta_row(tr(state, "Selection", "Selección"), summary))
    .child(row().gap(8.0).child(ghost_button(
        tr(state, "Edit crop", "Editar recorte"),
        &icons::CROP,
        |s: &mut State| s.edit_crop(),
    )));
    if current.is_some() {
        node = node.child(ghost_button(
            tr(state, "Clear crop", "Borrar recorte"),
            &icons::CLOSE,
            |s: &mut State| s.preference(Preference::Crop(None)),
        ));
    }
    node.child(
        components::label(
            tr(
                state,
                "A crop is stored as coordinates, so the original file is never rewritten.",
                "Un recorte se guarda como coordenadas, así el archivo original nunca se reescribe.",
            ),
            theme::MONO_SM,
            t.text_4,
        )
        .wrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    #[test]
    fn settings_scrolls_in_a_small_window() {
        let mut state = State {
            page: crate::ui::Page::Settings,
            ..State::default()
        };
        state.archive.preferences.language = Language::En;
        let mut runtime = Runtime::new(
            state,
            crate::ui::views::view,
            (1000.0, 760.0),
            Env::new(true, 1.0),
        );
        runtime.frame(0.0);
        let before = runtime.find("card-Desktop crop").expect("crop card").1;
        runtime.wheel(700.0, 500.0, -1200.0);
        for frame in 1..30 {
            runtime.frame(frame as f64 / 60.0);
        }
        let after = runtime
            .find("card-Desktop crop")
            .expect("crop card after scroll")
            .1;
        assert!(
            after < before - 20.0,
            "scroll did not move content: {before} -> {after}"
        );
        assert!(
            after < 760.0,
            "crop card remains outside the window: {after}"
        );
    }
}

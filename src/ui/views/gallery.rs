//! The archive: every capture this observatory kept, newest first.

use rune::{Cursor, El, ScrollSpec, SurfaceFit, col, el, row, spacer};
use solar_rust::domain::{Record, catalog};

use crate::ui::{
    State,
    components::{self, accent_button, ghost_button, meta_row},
    icons, theme,
};

use super::{Paint, format_bytes, page_header, picture, tr, utc_date, utc_hhmm};

pub fn view(state: &State, p: Paint) -> El {
    let mut records: Vec<&Record> = state.archive.records.iter().collect();
    records.sort_by_key(|r| std::cmp::Reverse(r.downloaded_at));

    let header = page_header(
        p,
        tr(state, "Operations / archive", "Operaciones / archivo"),
        tr(state, "The archive", "El archivo"),
        tr(
            state,
            "Local captures, kept under your retention rule. Nothing leaves this machine.",
            "Capturas locales, conservadas según tu regla de retención. Nada sale de esta máquina.",
        ),
        vec![
            ghost_button(
                tr(state, "Check now", "Buscar ahora"),
                &icons::SYNC,
                |s: &mut State| s.acquire(true, false),
            ),
            accent_button(
                tr(state, "Apply latest", "Aplicar última"),
                &icons::APPLY,
                |s: &mut State| s.apply_selected(),
            ),
        ],
    );

    let mut body = col().key("archive-body").w_full().gap(16.0);
    if let Some(latest) = records.first() {
        body = body
            .child(latest_card(state, p, latest))
            .child(history_controls(state, p, records.len()))
            .child(grid(state, p, &records));
    } else {
        body = body.child(empty(state, p));
    }

    col()
        .key("gallery-page")
        .w_full()
        .h_full()
        .min_h(0.0)
        .gap(16.0)
        .child(header)
        .child(clear_history_feedback(state, p))
        .child(body.h_full().min_h(0.0).scroll(ScrollSpec {
            stick_bottom: false,
        }))
}

fn history_controls(state: &State, p: Paint, count: usize) -> El {
    let t = p.t;
    let controls = if state.clear_history_confirming {
        row()
            .gap(8.0)
            .center_y()
            .child(components::label(
                tr(
                    state,
                    "Remove all stored capture history?",
                    "¿Eliminar todo el historial de capturas?",
                ),
                theme::BODY_SM,
                t.danger,
            ))
            .child(ghost_button(
                tr(state, "Cancel", "Cancelar"),
                &icons::CLOSE,
                |s: &mut State| s.cancel_clear_history(),
            ))
            .child(accent_button(
                tr(state, "Confirm clear", "Confirmar borrado"),
                &icons::CLOSE,
                |s: &mut State| s.confirm_clear_history(),
            ))
    } else if state.clear_history_pending {
        row().center_y().child(components::label(
            if state.clear_history_queued {
                tr(
                    state,
                    "Waiting for the current operation…",
                    "Esperando a que termine la operación actual…",
                )
            } else {
                tr(state, "Clearing history…", "Borrando historial…")
            },
            theme::BODY_SM,
            t.text_3,
        ))
    } else {
        row()
            .gap(12.0)
            .center_y()
            .child(components::label(
                format!("{count}"),
                theme::MONO_SM,
                t.text_4,
            ))
            .child(ghost_button(
                tr(state, "Clear history", "Borrar historial"),
                &icons::CLOSE,
                |s: &mut State| s.request_clear_history(),
            ))
    };
    row()
        .w_full()
        .center_y()
        .justify_between()
        .pt(6.0)
        .child(components::section_label(tr(
            state,
            "All captures",
            "Todas las capturas",
        )))
        .child(controls)
}

fn clear_history_feedback(state: &State, p: Paint) -> El {
    let Some(feedback) = &state.clear_history_feedback else {
        return el().h(0.0).w_full().a11y_hidden();
    };
    let (message, color) = match feedback {
        crate::ui::ClearHistoryFeedback::Cleared {
            preserved_files: _,
            active_unknown: true,
        } => (
            tr(
                state,
                "History cleared; capture files were retained because the active wallpaper could not be identified.",
                "Se borró el historial; se conservaron los archivos porque no se pudo identificar el fondo activo.",
            )
            .to_owned(),
            p.t.amber,
        ),
        crate::ui::ClearHistoryFeedback::Cleared {
            preserved_files: count,
            active_unknown: false,
        } if *count > 0 => (
            if state.archive.preferences.language == solar_rust::domain::Language::Es {
                format!(
                    "Se borró el historial; se conservaron {count} archivo{} para el fondo o el recorte.",
                    if *count == 1 { "" } else { "s" }
                )
            } else {
                format!(
                    "History cleared; {count} file{} were kept for wallpaper or crop settings.",
                    if *count == 1 { "" } else { "s" }
                )
            },
            p.t.amber,
        ),
        crate::ui::ClearHistoryFeedback::Cleared { .. } => (
            tr(state, "Capture history cleared.", "Se borró el historial de capturas.")
                .to_owned(),
            p.t.teal,
        ),
        crate::ui::ClearHistoryFeedback::Failed(error) => (
            format!(
                "{}: {error}",
                tr(state, "Could not clear history", "No se pudo borrar el historial")
            ),
            p.t.danger,
        ),
    };
    row()
        .w_full()
        .min_h(34.0)
        .px(10.0)
        .center_y()
        .radius(8.0)
        .fill(color.alpha(0.09))
        .border(1.0, color.alpha(0.22))
        .child(components::label(message, theme::BODY_SM, color).wrap())
}

fn latest_card(state: &State, p: Paint, record: &Record) -> El {
    let t = p.t;
    let channel = catalog::channel(&record.channel).ok();
    let id = record.id.clone();
    let preview = state.previews.get(&record.id);
    let name = channel
        .map(|c| c.name.clone())
        .unwrap_or_else(|| record.channel.clone());

    row()
        .key("latest-card")
        .w_full()
        // Keep the metadata and bottom action row inside the card at the standard window size.
        .h(304.0)
        .padding(10.0)
        .gap(16.0)
        .radius(18.0)
        .fill(t.panel)
        .border(1.0, t.hairline)
        .shadow(16.0, 42.0, rune::Color::BLACK.alpha(0.26))
        .child(
            el().grow(1.0)
                .h_full()
                .min_w(0.0)
                .radius(13.0)
                .clip_rounded(13.0)
                .fill(t.panel_sunken)
                .child(picture(preview, SurfaceFit::Fill, 13.0))
                .child(
                    el().pin(Some(14.0), Some(14.0), None, None)
                        .h(24.0)
                        .center_y()
                        .child(components::chip(
                            tr(state, "Newest", "Más reciente"),
                            t.amber,
                        )),
                ),
        )
        .child(
            col()
                .w(300.0)
                .h_full()
                .padding(8.0)
                .gap(9.0)
                .child(components::micro(
                    tr(state, "Latest capture", "Última captura"),
                    t.text_3,
                ))
                .child(components::label(name, theme::H1, t.text))
                .child(
                    components::label(
                        channel
                            .map(|c| {
                                c.description(state.archive.preferences.language)
                                    .to_string()
                            })
                            .unwrap_or_else(|| record.url.clone()),
                        theme::BODY_SM,
                        t.text_2,
                    )
                    .wrap()
                    .mb(2.0),
                )
                .child(components::hairline())
                .child(meta_row(
                    tr(state, "Captured", "Capturada"),
                    format!(
                        "{} {}",
                        utc_date(record.downloaded_at),
                        utc_hhmm(record.downloaded_at)
                    ),
                ))
                .child(meta_row(
                    tr(state, "Resolution", "Resolución"),
                    format!("{} x {}", record.width, record.height),
                ))
                .child(meta_row(
                    tr(state, "Size on disk", "Tamaño en disco"),
                    format_bytes(record.bytes),
                ))
                .child(spacer().grow(1.0))
                .child(
                    row()
                        .gap(8.0)
                        .child(
                            rune_kit::Button::new(tr(state, "Open", "Abrir"))
                                .variant(rune_kit::ButtonVariant::Neutral)
                                .icon(&icons::IMAGE)
                                .on_press(move |s: &mut State| s.select_image(id.clone()))
                                .build(p.k),
                        )
                        .child(ghost_button(
                            tr(state, "Set wallpaper", "Poner de fondo"),
                            &icons::APPLY,
                            |s: &mut State| s.apply_selected(),
                        )),
                ),
        )
}

fn grid(state: &State, p: Paint, records: &[&Record]) -> El {
    let mut cells = el().key("archive-grid").w_full().equal_columns(3).gap(14.0);
    for record in records {
        cells = cells.child(cell(state, p, record));
    }
    cells
}

fn cell(state: &State, p: Paint, record: &Record) -> El {
    let t = p.t;
    let preview = state.previews.get(&record.id);
    let id = record.id.clone();
    let name = catalog::channel(&record.channel)
        .map(|c| c.name.clone())
        .unwrap_or_else(|_| record.channel.clone());
    col()
        .key(format!("rec-{}", record.id))
        .gap(8.0)
        .padding(8.0)
        .radius(14.0)
        .fill(t.panel)
        .border(1.0, t.hairline)
        .hover(t.panel_hi)
        .press(t.glass)
        .focusable(true)
        .label(name.clone())
        .cursor(Cursor::Pointer)
        .on_press(move |s: &mut State| s.select_image(id.clone()))
        .child(
            el().w_full()
                .h(146.0)
                .radius(10.0)
                .clip_rounded(10.0)
                .fill(t.panel_sunken)
                .child(picture(preview, SurfaceFit::Fill, 10.0)),
        )
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .child(components::label(name, theme::LABEL, t.text).truncate())
                .child(components::label(
                    utc_hhmm(record.downloaded_at).replace(" UTC", ""),
                    theme::MONO_SM,
                    t.text_3,
                )),
        )
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .child(components::micro(utc_date(record.downloaded_at), t.text_4))
                .child(components::micro(format_bytes(record.bytes), t.text_4)),
        )
}

fn empty(state: &State, p: Paint) -> El {
    let t = p.t;
    col()
        .key("archive-empty")
        .w_full()
        .h(320.0)
        .center()
        .gap(10.0)
        .radius(18.0)
        .fill(t.panel)
        .border(1.0, t.hairline)
        .child(
            el().size(52.0, 52.0)
                .radius(26.0)
                .fill(t.glass_soft)
                .border(1.0, t.hairline)
                .center()
                .child(rune::icon(&icons::GALLERY, 22.0).color(t.text_3)),
        )
        .child(components::label(
            tr(state, "No captures yet", "Todavía no hay capturas"),
            theme::H2,
            t.text,
        ))
        .child(
            components::label(
                tr(
                    state,
                    "Run a check to download the latest image for the selected channel.",
                    "Ejecutá una búsqueda para descargar la última imagen del canal seleccionado.",
                ),
                theme::BODY_SM,
                t.text_3,
            )
            .wrap(),
        )
}

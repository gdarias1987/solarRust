//! Statistics: how much the archive holds, by channel and over time.

use std::collections::HashMap;

use rune::{El, ScrollSpec, SurfaceFit, col, el, row};
use solar_rust::domain::catalog;

use crate::ui::{
    State,
    components::{self, bar, meta_row, stat_tile},
    theme,
};

use super::{Paint, format_bytes, page_header, tr, utc_date, utc_hhmm};

pub fn view(state: &State, p: Paint) -> El {
    let t = p.t;
    let header = page_header(
        p,
        tr(state, "System / statistics", "Sistema / estadísticas"),
        tr(state, "Statistics", "Estadísticas"),
        tr(
            state,
            "What the archive holds, and where it came from.",
            "Qué contiene el archivo y de dónde vino.",
        ),
        vec![],
    );

    let total = state.archive.records.len();
    let bytes: u64 = state.archive.records.iter().map(|r| r.bytes).sum();
    let newest = state.archive.records.iter().map(|r| r.downloaded_at).max();
    let oldest = state.archive.records.iter().map(|r| r.downloaded_at).min();
    let mut per_channel: HashMap<&str, (usize, u64)> = HashMap::new();
    for record in &state.archive.records {
        let entry = per_channel.entry(record.channel.as_str()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += record.bytes;
    }
    let mut ranked: Vec<(&str, (usize, u64))> = per_channel.into_iter().collect();
    ranked.sort_by(|a, b| b.1.0.cmp(&a.1.0).then_with(|| a.0.cmp(b.0)));

    let tiles = row()
        .key("stat-tiles")
        .w_full()
        .gap(14.0)
        .child(stat_tile(
            tr(state, "Captures", "Capturas"),
            format!("{total}"),
            tr(state, "images", "imágenes"),
            t.teal,
        ))
        .child(stat_tile(
            tr(state, "Disk usage", "Uso de disco"),
            format_bytes(bytes),
            "",
            t.amber,
        ))
        .child(stat_tile(
            tr(state, "Channels", "Canales"),
            format!("{}", ranked.len()),
            tr(state, "of 19", "de 19"),
            t.violet,
        ));

    let by_channel = col()
        .key("by-channel")
        .grow(1.0)
        .min_w(0.0)
        .padding(16.0)
        .gap(13.0)
        .radius(16.0)
        .fill(t.panel)
        .border(1.0, t.hairline)
        .child(
            row()
                .w_full()
                .center_y()
                .justify_between()
                .child(components::section_label(tr(
                    state,
                    "By channel",
                    "Por canal",
                )))
                .child(components::micro(
                    if state.archive.preferences.language == solar_rust::domain::Language::Es {
                        format!("{} de 19 instrumentos", ranked.len())
                    } else {
                        format!("{} of 19 instruments", ranked.len())
                    },
                    t.text_4,
                )),
        )
        .children(if ranked.is_empty() {
            vec![empty_note(state, p)]
        } else {
            ranked
                .iter()
                .take(7)
                .map(|(id, (n, size))| {
                    let name = catalog::channel(id)
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|_| (*id).to_string());
                    col()
                        .key(format!("row-{id}"))
                        .w_full()
                        .gap(6.0)
                        .child(
                            row()
                                .w_full()
                                .center_y()
                                .justify_between()
                                .child(components::label(name, theme::BODY_SM, t.text_2))
                                .child(components::label(
                                    format!("{n} · {}", format_bytes(*size)),
                                    theme::MONO_SM,
                                    t.text_3,
                                )),
                        )
                        .child(bar(*n as f32 / total.max(1) as f32, t.teal))
                })
                .collect()
        })
        .child(components::hairline())
        .child(components::micro(
            tr(state, "Coverage", "Cobertura"),
            t.text_3,
        ))
        .child(coverage(state, p))
        .child(components::hairline())
        .child(meta_row(
            tr(state, "Newest capture", "Captura más reciente"),
            newest
                .map(|at| format!("{} {}", utc_date(at), utc_hhmm(at)))
                .unwrap_or_else(|| "—".into()),
        ))
        .child(meta_row(
            tr(state, "Oldest capture", "Captura más antigua"),
            oldest.map(utc_date).unwrap_or_else(|| "—".into()),
        ))
        .child(meta_row(
            tr(state, "Retention rule", "Regla de retención"),
            format!(
                "{} {}",
                state.archive.preferences.retention,
                tr(state, "images", "imágenes")
            ),
        ));

    let recent = col()
        .key("recent")
        .w(340.0)
        .padding(16.0)
        .gap(10.0)
        .radius(16.0)
        .fill(t.panel)
        .border(1.0, t.hairline)
        .child(components::section_label(tr(
            state,
            "Recent captures",
            "Capturas recientes",
        )))
        .children(recent_rows(state, p));

    col()
        .key("statistics-page")
        .w_full()
        .h_full()
        .min_h(0.0)
        .gap(16.0)
        .child(header)
        .child(tiles)
        .child(
            row()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .gap(14.0)
                .child(by_channel)
                .child(recent.scroll(ScrollSpec {
                    stick_bottom: false,
                })),
        )
        .child(operation(state, p))
}

fn recent_rows(state: &State, p: Paint) -> Vec<rune::El> {
    let t = p.t;
    let mut records: Vec<&solar_rust::domain::Record> = state.archive.records.iter().collect();
    records.sort_by_key(|r| std::cmp::Reverse(r.downloaded_at));
    if records.is_empty() {
        return vec![empty_note(state, p)];
    }
    records
        .into_iter()
        .take(8)
        .map(|record| {
            let name = catalog::channel(&record.channel)
                .map(|c| c.name.clone())
                .unwrap_or_else(|_| record.channel.clone());
            row()
                .key(format!("recent-{}", record.id))
                .w_full()
                .h(38.0)
                .center_y()
                .gap(10.0)
                .child(
                    el().size(26.0, 26.0)
                        .radius(7.0)
                        .clip_rounded(7.0)
                        .fill(t.panel_sunken)
                        .child(super::picture(
                            state.previews.get(&format!("ch:{}", record.channel)),
                            SurfaceFit::Fill,
                            7.0,
                        )),
                )
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(1.0)
                        .child(components::label(name, theme::LABEL, t.text).truncate())
                        .child(components::micro(
                            format!(
                                "{} · {}",
                                utc_date(record.downloaded_at),
                                utc_hhmm(record.downloaded_at)
                            ),
                            t.text_4,
                        )),
                )
                .child(components::label(
                    format_bytes(record.bytes),
                    theme::MONO_SM,
                    t.text_3,
                ))
        })
        .collect()
}

/// One cell per instrument: filled when the archive holds a capture from it.
fn coverage(state: &State, p: Paint) -> El {
    let t = p.t;
    let mut cells = row().w_full().flex_wrap().gap(5.0);
    for channel in catalog::channels() {
        let captured = state
            .archive
            .records
            .iter()
            .any(|r| r.channel == channel.id);
        cells = cells.child(
            el().key(format!("cov-{}", channel.id))
                .size(14.0, 14.0)
                .radius(4.0)
                .fill(if captured {
                    t.teal.alpha(0.72)
                } else {
                    t.glass_soft
                })
                .border(
                    1.0,
                    if captured {
                        t.teal.alpha(0.9)
                    } else {
                        t.hairline
                    },
                )
                .help(channel.name.clone())
                .a11y_hidden(),
        );
    }
    cells
}

fn empty_note(state: &State, p: Paint) -> rune::El {
    components::label(
        tr(
            state,
            "No captures yet. Run a check from the channel page.",
            "Todavía no hay capturas. Ejecutá una búsqueda desde la página de canales.",
        ),
        theme::BODY_SM,
        p.t.text_3,
    )
    .wrap()
}

fn operation(state: &State, p: Paint) -> rune::El {
    let t = p.t;
    let mut line = row()
        .key("last-operation")
        .w_full()
        .center_y()
        .gap(10.0)
        .child(components::micro(
            tr(state, "Last operation", "Última operación"),
            t.text_4,
        ));
    match &state.outcome {
        Some(outcome) => {
            line = line
                .child(components::label(
                    outcome.channel.clone(),
                    theme::MONO,
                    t.text_2,
                ))
                .child(stage_chip(
                    state,
                    tr(state, "Check", "Chequeo"),
                    &outcome.check,
                    p,
                ))
                .child(stage_chip(
                    state,
                    tr(state, "Download", "Descarga"),
                    &outcome.download,
                    p,
                ))
                .child(stage_chip(
                    state,
                    tr(state, "Store", "Guardado"),
                    &outcome.persistence,
                    p,
                ))
                .child(stage_chip(
                    state,
                    tr(state, "Apply", "Aplicar"),
                    &outcome.apply,
                    p,
                ));
        }
        None => {
            line = line.child(components::label(
                tr(
                    state,
                    "Nothing yet this session",
                    "Nada todavía en esta sesión",
                ),
                theme::MONO_SM,
                t.text_4,
            ));
        }
    }
    line
}

fn stage_chip(
    _state: &State,
    label: &str,
    stage: &solar_rust::application::Stage,
    p: Paint,
) -> rune::El {
    use solar_rust::application::Stage;
    let t = p.t;
    let (mark, ink) = match stage {
        Stage::Succeeded => ("ok", t.success),
        Stage::Unchanged => ("same", t.text_3),
        Stage::Failed(_) => ("error", t.danger),
        Stage::NotRun => ("—", t.text_4),
    };
    row()
        .gap(5.0)
        .center_y()
        .child(components::micro(label, t.text_4))
        .child(components::label(mark, theme::MONO_SM, ink))
}

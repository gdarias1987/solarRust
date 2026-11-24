//! Channels: the instruments, the latest capture, and the grid that picks one.

use rune::{Cursor, El, ScrollSpec, SurfaceFit, col, el, row, spacer};
use solar_rust::domain::{catalog, catalog::Channel};

use crate::ui::{
    State,
    components::{self, accent_button, chip, ghost_button, meta_row},
    icons, theme,
};

use super::{Paint, page_header, picture, temperature, temperature_equivalent, tr};

const CATEGORIES: [&str; 4] = ["All", "AIA", "HMI", "Composite"];
/// The narrowest an instrument card gets; past that the grid adds a column. A row always fills
/// the width it was measured at, so cards share the leftover points between them.
const CARD: f32 = 146.0;
const CARD_GAP: f32 = 10.0;

pub fn view(state: &State, p: Paint, cx: &mut rune::Cx<State>) -> El {
    let id = state.selected_channel().to_owned();
    let active = catalog::channel(&id).ok();
    let header = page_header(
        p,
        tr(state, "Operations / channels", "Operaciones / canales"),
        tr(state, "Channel operations", "Operación de canales"),
        tr(
            state,
            "Nineteen instruments over one archive. Pick a wavelength, inspect the newest capture, then put the Sun on your desktop.",
            "Diecinueve instrumentos sobre un archivo. Elegí una longitud de onda, revisá la última captura y poné el Sol en tu escritorio.",
        ),
        vec![ghost_button(
            tr(state, "Check now", "Buscar ahora"),
            &icons::SYNC,
            |s: &mut State| s.acquire(true, false),
        )],
    );
    col()
        .key("channels-page")
        .w_full()
        .h_full()
        .min_h(0.0)
        .gap(16.0)
        .child(header)
        // A short window scrolls the decks vertically: the grid keeps the width it has instead
        // of growing a horizontal scrollbar.
        .child(
            col()
                .key("channels-body")
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .gap(16.0)
                .child(hero(state, p, active))
                .child(grid_header(state, p))
                .child(instrument_grid(state, p, cx))
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                // When the window is too short for the grid, the bottom edge fades so the cut
                // row reads as "there is more below" instead of a bug.
                .scroll_edge_fade(28.0),
        )
}

fn hero(state: &State, p: Paint, active: Option<&'static Channel>) -> El {
    let t = p.t;
    let id = active.map(|c| c.id.as_str()).unwrap_or("0304");
    let preview = channel_capture_preview(state, id);
    let has_capture = state
        .archive
        .latest(id)
        .is_some_and(|record| state.previews.contains_key(&record.id));
    let name = active
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "AIA 304".into());
    let language = state.archive.preferences.language;
    let description = active
        .map(|c| c.description(language).to_string())
        .unwrap_or_else(|| tr(state, "No channel selected.", "Ningún canal seleccionado.").into());
    let wavelength = active.map(|c| c.wavelength.clone()).unwrap_or_default();
    let ions = active.map(|c| c.ions.clone()).unwrap_or_default();
    let region = active
        .map(|c| c.region(language).to_string())
        .unwrap_or_default();
    let temperatures = active.map(temperature).unwrap_or_default();
    let archived = active
        .map(|c| {
            state
                .archive
                .records
                .iter()
                .filter(|r| r.channel == c.id)
                .count()
        })
        .unwrap_or(0);
    let archive_badge = match (archived, state.archive.preferences.language) {
        (0, _) => tr(state, "No captures", "Sin capturas").to_string(),
        (1, solar_rust::domain::Language::Es) => "1 captura".into(),
        (1, _) => "1 capture".into(),
        (n, solar_rust::domain::Language::Es) => format!("{n} capturas"),
        (n, _) => format!("{n} captures"),
    };

    // The disc is the subject: a square frame shows all of it, never a crop.
    let frame = el()
        .key("hero-frame")
        .w(360.0)
        .h(360.0)
        .radius(18.0)
        .clip_rounded(18.0)
        .fill(t.panel_sunken)
        .border(1.0, t.hairline)
        .shadow(0.0, 44.0, t.amber.alpha(0.14))
        .shadow(0.0, 10.0, rune::Color::BLACK.alpha(0.45))
        .child(picture(preview, SurfaceFit::Fit, 18.0))
        .child(
            el().pin(Some(0.0), None, Some(0.0), Some(0.0))
                .h(104.0)
                .linear_gradient(
                    std::f32::consts::PI,
                    [
                        (0.0, rune::Color::TRANSPARENT),
                        (1.0, rune::Color::BLACK.alpha(0.78)),
                    ],
                )
                .a11y_hidden(),
        )
        .child(
            row()
                .pin(Some(16.0), None, Some(16.0), Some(16.0))
                .h(24.0)
                .center_y()
                .justify_between()
                .child(
                    row()
                        .gap(8.0)
                        .center_y()
                        .child(components::status_dot(t.amber, true))
                        .child(components::micro(
                            if has_capture {
                                tr(state, "Latest capture", "Última captura")
                            } else {
                                tr(state, "Reference image", "Imagen de referencia")
                            },
                            t.text,
                        )),
                )
                .child(components::micro(archive_badge, t.text_2)),
        )
        .child(
            el().pin(Some(16.0), Some(16.0), Some(16.0), None)
                .h(22.0)
                .child(chip(
                    if state.demo_mode {
                        tr(state, "Demo fixture", "Fixture demo")
                    } else {
                        "SDO / NASA"
                    },
                    t.teal,
                )),
        )
        // A restrained liquid-metal bezel frames the selected instrument's focal solar image.
        .child(
            el().key("hero-metal-bezel")
                .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .radius(18.0)
                .effect(super::super::effects::METAL),
        );

    let simple = active
        .map(|c| c.simple(language).to_string())
        .unwrap_or_default();
    let kids = active
        .map(|c| c.kids(language).to_string())
        .unwrap_or_default();
    let scale = active
        .map(|c| temperature_equivalent(c, language))
        .unwrap_or_default();

    row()
        .key("hero-row")
        .w_full()
        .gap(16.0)
        .center_y()
        .child(frame)
        .child(
            col()
                .key("hero-panel")
                .grow(1.0)
                .min_w(0.0)
                .padding(18.0)
                .gap(4.0)
                .radius(18.0)
                .fill(t.panel)
                .border(1.0, t.hairline)
                .shadow(16.0, 40.0, rune::Color::BLACK.alpha(0.28))
                .child(components::micro(
                    tr(state, "Selected channel", "Canal seleccionado"),
                    t.text_3,
                ))
                .child(components::label(name, theme::H1, t.text))
                .child(components::label(description, theme::BODY, t.text_2).wrap())
                .child(components::hairline())
                .child(meta_row(
                    tr(state, "Wavelength", "Longitud de onda"),
                    wavelength,
                ))
                .child(meta_row(tr(state, "Ion", "Ion"), ions))
                .child(meta_row(tr(state, "Region", "Región"), region))
                .child(meta_row(
                    tr(state, "Temperature", "Temperatura"),
                    format!("{temperatures}  ·  {scale}"),
                ))
                .child(components::hairline())
                .child(
                    col()
                        .w_full()
                        .gap(3.0)
                        .child(components::micro(
                            tr(state, "In plain words", "En palabras simples"),
                            t.text_3,
                        ))
                        .child(components::label(simple, theme::BODY_SM, t.text_2).wrap()),
                )
                .child(
                    col()
                        .w_full()
                        .padding(10.0)
                        .gap(3.0)
                        .radius(10.0)
                        .fill(t.violet.alpha(0.09))
                        .border(1.0, t.violet.alpha(0.22))
                        .child(components::micro(
                            tr(state, "For young astronomers", "Para pequeños astrónomos"),
                            t.violet,
                        ))
                        .child(components::label(kids, theme::BODY_SM, t.text_2).wrap()),
                )
                .child(spacer().h(4.0))
                .child(
                    row()
                        .w_full()
                        .gap(8.0)
                        .child(accent_button(
                            match state.apply_feedback {
                                Some(crate::ui::ApplyFeedback::Applying) => {
                                    tr(state, "Applying…", "Aplicando…")
                                }
                                Some(crate::ui::ApplyFeedback::Applied) => {
                                    tr(state, "Applied", "Aplicado")
                                }
                                None => tr(state, "Set wallpaper", "Poner de fondo"),
                            },
                            &icons::APPLY,
                            |s: &mut State| s.apply_selected(),
                        ))
                        .child(ghost_button(
                            tr(state, "Crop", "Recortar"),
                            &icons::CROP,
                            |s: &mut State| s.edit_crop(),
                        ))
                        .child(match state.apply_feedback {
                            Some(crate::ui::ApplyFeedback::Applying) => row()
                                .key("wallpaper-applying")
                                .gap(6.0)
                                .center_y()
                                .child(components::pulse_dot(t.teal, 0.8, true))
                                .child(components::micro(
                                    tr(state, "Applying wallpaper…", "Aplicando el fondo…"),
                                    t.teal,
                                )),
                            Some(crate::ui::ApplyFeedback::Applied) => row()
                                .key("wallpaper-applied")
                                .gap(6.0)
                                .center_y()
                                .child(components::status_dot(t.teal, false))
                                .child(components::micro(
                                    tr(state, "Wallpaper applied", "Fondo aplicado"),
                                    t.teal,
                                )),
                            None => el(),
                        }),
                ),
        )
}

fn grid_header(state: &State, p: Paint) -> El {
    let t = p.t;
    let categories: [&str; 4] =
        if state.archive.preferences.language == solar_rust::domain::Language::Es {
            ["Todos", "AIA", "HMI", "Composite"]
        } else {
            CATEGORIES
        };
    let total = catalog::channels()
        .iter()
        .filter(|c| matches_filter(c, state.filter))
        .count();
    row()
        .key("grid-header")
        .w_full()
        .center_y()
        .justify_between()
        .child(
            row()
                .gap(12.0)
                .center_y()
                .child(components::section_label(tr(
                    state,
                    "Instruments",
                    "Instrumentos",
                )))
                .child(components::label(
                    format!("{total}"),
                    theme::MONO_SM,
                    t.text_4,
                )),
        )
        .child(
            rune_kit::SegmentedControl::new(&categories, Some(state.filter))
                .fit()
                .label(tr(state, "Filter", "Filtro"))
                .on_pick(|s: &mut State, index, _| s.filter = index)
                .build(p.k),
        )
}

fn matches_filter(channel: &Channel, filter: usize) -> bool {
    match filter {
        1 => channel.category == "AIA",
        2 => channel.category == "HMI",
        3 => channel.category == "Composite",
        _ => true,
    }
}

/// Instruments as a desktop grid: the column count comes from the body's measured width, so a
/// wide window shows more columns and a narrow one wraps to another row. The page never grows a
/// horizontal scrollbar, because every row is laid out to fit the width it was measured at.
fn instrument_grid(state: &State, p: Paint, cx: &mut rune::Cx<State>) -> El {
    let (columns, width) = columns_for(cx.layout_with("channels-body", |rect| rect[2]));
    let mut grid = col().key("instrument-grid").w_full().gap(CARD_GAP);
    for chunk in catalog::channels()
        .iter()
        .filter(|channel| matches_filter(channel, state.filter))
        .collect::<Vec<_>>()
        .chunks(columns)
    {
        let mut line = row().w_full().gap(CARD_GAP);
        for channel in chunk {
            line = line.child(card(state, p, channel, width));
        }
        grid = grid.child(line);
    }
    grid
}

/// How many cards fit across `width`, and how wide each one has to be to fill the row. Six
/// columns of the base width are the guess before the first layout reports a width.
fn columns_for(width: Option<f32>) -> (usize, f32) {
    let Some(width) = width.filter(|width| width.is_finite() && *width > 0.0) else {
        return (6, CARD);
    };
    let columns = (((width + CARD_GAP) / (CARD + CARD_GAP)).floor() as usize).max(1);
    let gaps = CARD_GAP * (columns - 1) as f32;
    (columns, ((width - gaps) / columns as f32).floor())
}

fn channel_capture_preview<'a>(state: &'a State, id: &str) -> Option<&'a rune::Pixels> {
    state
        .archive
        .latest(id)
        .and_then(|record| state.previews.get(&record.id))
        .or_else(|| state.previews.get(&format!("ch:{id}")))
        .or_else(|| state.reference_previews.get(id))
}

fn channel_thumbnail<'a>(state: &'a State, id: &str) -> Option<&'a rune::Pixels> {
    state.reference_previews.get(id)
}

fn card(state: &State, p: Paint, channel: &'static Channel, width: f32) -> El {
    let t = p.t;
    let selected = state.selected_channel() == channel.id;
    let preview = channel_thumbnail(state, &channel.id);
    let id = channel.id.clone();
    // The padding is 8 a side, so a square of the inner width keeps the whole Sun in the tile
    // instead of cropping a band out of it.
    let thumb = (width - 16.0).max(0.0);
    let mut node = col()
        .key(format!("ch-{}", channel.id))
        .w(width)
        .shrink()
        .padding(8.0)
        .gap(7.0)
        .radius(13.0)
        .fill(if selected { t.panel_hi } else { t.panel })
        .border(
            1.0,
            if selected {
                t.teal.alpha(0.45)
            } else {
                t.hairline
            },
        )
        .hover(t.panel_hi)
        .motion(rune::Spring::new(0.26, 0.0))
        .focusable(true)
        .label(channel.name.clone())
        .cursor(Cursor::Pointer)
        .on_press(move |s: &mut State| s.select_channel(id.clone()))
        .child(
            el().w_full()
                .h(thumb)
                .radius(9.0)
                .clip_rounded(9.0)
                .fill(t.panel_sunken)
                .child(picture(preview, SurfaceFit::Fill, 9.0)),
        )
        .child(
            components::label(channel.name.clone(), theme::LABEL, t.text)
                .truncate()
                .w_full(),
        )
        // Both lines are one line, cut to the tile: a long wavelength never spills past the
        // card's border, and every card keeps the same height.
        .child(
            components::micro(channel.wavelength.clone(), t.text_4)
                .truncate()
                .w_full(),
        );
    if selected {
        node = node.shadow(0.0, 14.0, t.teal.alpha(0.18));
    }
    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    #[test]
    fn channel_capture_preview_uses_latest_archived_capture() {
        let mut state = State::default();
        let record = |id: &str, time| solar_rust::domain::Record {
            id: id.into(),
            channel: "0304".into(),
            url: String::new(),
            remote_modified: None,
            downloaded_at: time,
            bytes: 0,
            file: String::new(),
            width: 1,
            height: 1,
        };
        state.archive.records = vec![record("old", 1), record("new", 2)];
        state.previews.insert("old".into(), rune::Pixels::new());
        assert!(std::ptr::eq(
            channel_capture_preview(&state, "0304").unwrap(),
            &state.reference_previews["0304"]
        ));
        state.previews.insert("new".into(), rune::Pixels::new());
        assert!(std::ptr::eq(
            channel_capture_preview(&state, "0304").unwrap(),
            &state.previews["new"]
        ));
        assert!(channel_capture_preview(&state, "missing").is_none());
    }

    #[test]
    fn channel_thumbnails_always_use_the_catalog_reference() {
        let mut state = State::default();
        let record = solar_rust::domain::Record {
            id: "latest-capture".into(),
            channel: "0304".into(),
            url: String::new(),
            remote_modified: None,
            downloaded_at: 1,
            bytes: 0,
            file: String::new(),
            width: 1,
            height: 1,
        };
        state.archive.records.push(record);
        state
            .previews
            .insert("latest-capture".into(), rune::Pixels::new());

        assert!(std::ptr::eq(
            channel_thumbnail(&state, "0304").unwrap(),
            &state.reference_previews["0304"]
        ));
        assert!(channel_thumbnail(&state, "missing").is_none());
    }

    #[test]
    fn hero_has_the_single_channel_apply_action() {
        let (commands, receive) = std::sync::mpsc::channel();
        let mut state = State {
            commands: Some(commands),
            ..State::default()
        };
        state.archive.records.push(solar_rust::domain::Record {
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
        let mut runtime = Runtime::new(
            state,
            crate::ui::views::view,
            (1180.0, 820.0),
            Env::new(true, 1.0),
        );
        runtime.frame(0.0);
        assert!(runtime.find("accent-Aplicar").is_none());
        let (x, y) = runtime
            .find("accent-Poner de fondo")
            .expect("hero apply button");
        runtime.pointer_move(x, y);
        runtime.pointer_down(x, y);
        runtime.pointer_up(x, y);
        runtime.frame(0.0);

        assert!(matches!(
            receive.try_recv(),
            Ok(crate::ui::worker::Command::Apply { id, .. }) if id == "capture-1"
        ));
    }

    #[test]
    fn instrument_grid_wraps_within_the_body_width() {
        let state = State::default();
        let mut rt = Runtime::new(
            state,
            crate::ui::views::view,
            (1000.0, 760.0),
            Env::new(true, 1.0),
        );
        rt.frame(0.0);
        let elements = rt.elements();
        let body = elements
            .iter()
            .find(|e| e.key.as_deref() == Some("channels-body"))
            .expect("channels body")
            .rect;
        let keys: Vec<String> = catalog::channels()
            .iter()
            .map(|c| format!("ch-{}", c.id))
            .collect();
        let cards: Vec<(&str, [f32; 4])> = elements
            .iter()
            .filter_map(|e| {
                let key = e.key.as_deref()?;
                keys.iter().any(|k| k == key).then_some((key, e.rect))
            })
            .collect();
        assert_eq!(cards.len(), catalog::channels().len(), "grid lost channels");
        // Every card stays inside the body: the grid wraps instead of scrolling sideways.
        for (key, rect) in &cards {
            assert!(
                rect[0] + rect[2] <= body[0] + body[2] + 0.5,
                "{key} overflows the body: {rect:?} in {body:?}"
            );
        }
        // At this width the grid uses several columns and wraps to a second row.
        let top = cards[0].1[1];
        let first_row = cards
            .iter()
            .filter(|(_, rect)| (rect[1] - top).abs() < 0.5)
            .count();
        assert!(first_row >= 2, "cards did not share a row: {first_row}");
        assert!(
            cards.iter().any(|(_, rect)| rect[1] > top + 0.5),
            "grid never wrapped"
        );
        // One size for every tile, and its type stays inside it: a long channel name or a
        // composite wavelength is cut, never painted over the neighbour.
        let (width, height) = (cards[0].1[2], cards[0].1[3]);
        let texts: Vec<[f32; 4]> = elements
            .iter()
            .filter(|e| e.role == "text")
            .map(|e| e.rect)
            .collect();
        for (key, rect) in &cards {
            assert!(
                (rect[2] - width).abs() < 0.5 && (rect[3] - height).abs() < 0.5,
                "{key} changed size: {rect:?}"
            );
            for text in &texts {
                let inside = text[0] >= rect[0] - 0.5
                    && text[0] < rect[0] + rect[2]
                    && text[1] >= rect[1] - 0.5
                    && text[1] < rect[1] + rect[3];
                if inside {
                    assert!(
                        text[0] + text[2] <= rect[0] + rect[2] + 0.5,
                        "{key} spills text past its border: {text:?} in {rect:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_filtered_grid_keeps_the_same_card_width() {
        let state = State {
            filter: 2,
            ..State::default()
        };
        let expected = catalog::channels()
            .iter()
            .filter(|c| matches_filter(c, 2))
            .map(|c| format!("ch-{}", c.id))
            .collect::<Vec<_>>();
        let mut rt = Runtime::new(
            state,
            crate::ui::views::view,
            (1000.0, 760.0),
            Env::new(true, 1.0),
        );
        rt.frame(0.0);
        let elements = rt.elements();
        let cards: Vec<[f32; 4]> = elements
            .iter()
            .filter(|e| {
                e.key
                    .as_deref()
                    .is_some_and(|key| expected.iter().any(|k| k == key))
            })
            .map(|e| e.rect)
            .collect();
        assert_eq!(
            cards.len(),
            expected.len(),
            "filter kept the wrong channels"
        );
        let body = elements
            .iter()
            .find(|e| e.key.as_deref() == Some("channels-body"))
            .expect("channels body")
            .rect;
        // The filter changes the count, not the row: cards keep one width, each row starts at the
        // grid's left edge, and nothing spills past the body.
        let width = cards[0][2];
        let left = cards[0][0];
        let mut rows: Vec<f32> = Vec::new();
        for rect in &cards {
            assert!(
                (rect[2] - width).abs() < 0.5,
                "card widths drifted: {rect:?}"
            );
            assert!(rect[0] >= left - 0.5, "card left of the grid: {rect:?}");
            assert!(
                rect[0] + rect[2] <= body[0] + body[2] + 0.5,
                "card overflows: {rect:?}"
            );
            if !rows.iter().any(|row| (row - rect[1]).abs() < 0.5) {
                rows.push(rect[1]);
            }
        }
        for row in &rows {
            let first = cards
                .iter()
                .filter(|rect| (rect[1] - row).abs() < 0.5)
                .map(|rect| rect[0])
                .fold(f32::INFINITY, f32::min);
            assert!(
                (first - left).abs() < 0.5,
                "row does not start at the grid's left: {row}"
            );
        }
        assert!(rows.len() >= 2, "filtered grid did not wrap");
    }
}

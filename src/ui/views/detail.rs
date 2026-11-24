//! One capture, large: the inspection surface.

use rune::{El, SurfaceFit, col, el, row, spacer};
use solar_rust::domain::{Record, catalog};

use crate::ui::{
    Page, State,
    components::{self, accent_button, ghost_button, meta_row},
    icons, theme,
};

use super::{Paint, format_bytes, picture, tr, utc_date, utc_hhmm};

pub fn view(state: &State, p: Paint) -> El {
    let t = p.t;
    let record = state.image();
    let channel = record.and_then(|r| catalog::channel(&r.channel).ok());
    let name = channel
        .map(|c| c.name.clone())
        .unwrap_or_else(|| tr(state, "No capture", "Sin captura").into());

    let top = row()
        .key("detail-top")
        .w_full()
        .center_y()
        .gap(12.0)
        .child(components::icon_button(
            &icons::ARROW_LEFT,
            tr(state, "Back", "Volver"),
            |s: &mut State| s.go(Page::Gallery),
        ))
        .child(super::chrome::breadcrumb(&[
            tr(state, "Archive", "Archivo"),
            &name,
        ]))
        .child(spacer().grow(1.0))
        .child(ghost_button(
            tr(state, "Crop", "Recortar"),
            &icons::CROP,
            |s: &mut State| s.edit_crop(),
        ))
        .child(accent_button(
            tr(state, "Set wallpaper", "Poner de fondo"),
            &icons::APPLY,
            |s: &mut State| s.apply_selected(),
        ));

    let image = el()
        .key("detail-image")
        .grow(1.0)
        .h_full()
        .min_w(0.0)
        .radius(18.0)
        .clip_rounded(18.0)
        .fill(t.panel_sunken)
        .border(1.0, t.hairline)
        .child(picture(
            record.and_then(|r| state.previews.get(&r.id)),
            SurfaceFit::Fit,
            18.0,
        ))
        .child(
            el().pin(Some(14.0), None, None, Some(14.0))
                .h(24.0)
                .child(components::chip(
                    record
                        .map(|r| format!("{} x {}", r.width, r.height))
                        .unwrap_or_else(|| "—".into()),
                    t.teal,
                )),
        );

    col()
        .key("detail-page")
        .w_full()
        .h_full()
        .min_h(0.0)
        .gap(16.0)
        .child(top)
        .child(
            row()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .gap(16.0)
                .child(image)
                .child(details(state, p, record, channel)),
        )
}

fn details(
    state: &State,
    p: Paint,
    record: Option<&Record>,
    channel: Option<&'static catalog::Channel>,
) -> El {
    let t = p.t;
    let mut panel = col()
        .key("detail-panel")
        .w(316.0)
        .h_full()
        .min_h(0.0)
        .padding(18.0)
        .gap(9.0)
        .radius(18.0)
        // Leave enough translucency for the glass slot to sample what is behind this panel.
        // SolarFx paints an opaque static fallback at the slot if backdrop copies are unavailable.
        .fill(t.panel.alpha(0.72))
        .border(1.0, t.hairline)
        .scroll(rune::ScrollSpec {
            stick_bottom: false,
        })
        .child(
            el().key("detail-glass-slot")
                .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .effect(super::super::effects::GLASS)
                .effect_backdrop(true),
        )
        .child(components::micro(tr(state, "Capture", "Captura"), t.text_3))
        .child(components::label(
            channel
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "—".into()),
            theme::H1,
            t.text,
        ))
        .child(
            components::label(
                channel
                    .map(|c| {
                        c.description(state.archive.preferences.language)
                            .to_string()
                    })
                    .unwrap_or_else(|| "—".into()),
                theme::BODY_SM,
                t.text_2,
            )
            .wrap()
            .mb(2.0),
        )
        .child(components::hairline());

    let meta = match record {
        Some(r) => vec![
            (
                tr(state, "Captured", "Capturada"),
                format!(
                    "{} {}",
                    utc_date(r.downloaded_at),
                    utc_hhmm(r.downloaded_at)
                ),
            ),
            (
                tr(state, "Resolution", "Resolución"),
                format!("{} x {}", r.width, r.height),
            ),
            (tr(state, "Size", "Tamaño"), format_bytes(r.bytes)),
            (tr(state, "Channel", "Canal"), r.channel.clone()),
            (
                tr(state, "Ion", "Ion"),
                channel.map(|c| c.ions.clone()).unwrap_or_default(),
            ),
            (
                tr(state, "Wavelength", "Longitud de onda"),
                channel.map(|c| c.wavelength.clone()).unwrap_or_default(),
            ),
            (
                tr(state, "Region", "Región"),
                channel
                    .map(|c| c.region(state.archive.preferences.language).to_string())
                    .unwrap_or_default(),
            ),
            (
                tr(state, "Temperature", "Temperatura"),
                channel.map(super::temperature).unwrap_or_default(),
            ),
        ],
        None => Vec::new(),
    };
    for (key, value) in meta {
        // Prose values (regions) get their own line; short ones stay in the key/value row.
        panel = panel.child(if value.chars().count() > 22 {
            components::stack_row(key, value)
        } else {
            meta_row(key, value)
        });
    }

    let file = record.map(|r| r.file.clone()).unwrap_or_default();
    panel
        .child(components::hairline())
        .child(components::micro(tr(state, "Local file", "Archivo local"), t.text_3))
        .child(
            components::label(file, theme::MONO_SM, t.text_2)
                .wrap()
                .mb(2.0),
        )
        .child(spacer().grow(1.0))
        .child(components::label(
            tr(
                state,
                "Original resolution is preserved; the crop is stored as coordinates, never as a second file.",
                "La resolución original se conserva; el recorte se guarda como coordenadas, nunca como un segundo archivo.",
            ),
            theme::MONO_SM,
            t.text_4,
        )
        .wrap())
}

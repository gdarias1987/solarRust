//! The crop editor: an overlay that frames the part of the original that becomes the desktop.

use rune::{Cursor, Drag, Phase, SurfaceFit, col, el, row, spacer};

use crate::ui::{
    State,
    components::{self, accent_button, ghost_button},
    icons, theme,
};

use super::{Paint, tr};

/// The design box the crop frame is drawn into, in points.
const IMG_W: f32 = 760.0;
const IMG_H: f32 = 400.0;

pub fn overlay(state: &State, p: Paint, _frame: Option<[f32; 2]>) -> rune::El {
    let t = p.t;
    let record = state.image();
    let crop = state.draft.as_ref();
    let source = record.cloned();
    let (orig_w, orig_h) = crop
        .map(|c| (c.original_width as f32, c.original_height as f32))
        .or_else(|| source.as_ref().map(|r| (r.width as f32, r.height as f32)))
        .unwrap_or((1.0, 1.0));
    let preview = record.and_then(|r| state.previews.get(&r.id));

    // Where the image actually lands inside the box when fitted, so the frame tracks it.
    let (pw, ph) = preview
        .map(|px| {
            let (w, h) = px.size();
            (w as f32, h as f32)
        })
        .unwrap_or((orig_w, orig_h));
    let scale = (IMG_W / pw).min(IMG_H / ph);
    let (dw, dh) = (pw * scale, ph * scale);
    let (ox, oy) = ((IMG_W - dw) / 2.0, (IMG_H - dh) / 2.0);

    let mut stage = el()
        .key("crop-stage")
        .w(IMG_W)
        .h(IMG_H)
        .radius(12.0)
        .clip_rounded(12.0)
        .fill(rune::Color::BLACK)
        .border(1.0, t.hairline)
        .cursor(Cursor::Grab)
        .on_drag(move |s: &mut State, d: &mut Drag, _tx| {
            let (sx, sy) = (orig_w / dw.max(1.0), orig_h / dh.max(1.0));
            match d.phase {
                Phase::Down => {
                    s.crop_drag = Some((0.0, 0.0));
                    d.data = 0.0;
                }
                Phase::Move => {
                    let last = s.crop_drag.unwrap_or((0.0, 0.0));
                    s.drag_crop(((d.dx - last.0) * sx) as f64, ((d.dy - last.1) * sy) as f64);
                    s.crop_drag = Some((d.dx, d.dy));
                }
                Phase::Up => {
                    s.crop_drag = None;
                    d.data = 0.0;
                }
            }
        })
        .child(
            el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .child(super::picture(preview, SurfaceFit::Fit, 12.0)),
        );

    if let Some(crop) = crop {
        let left = ox + (crop.x as f32 / orig_w) * dw;
        let top = oy + (crop.y as f32 / orig_h) * dh;
        let width = (crop.width as f32 / orig_w) * dw;
        let height = (crop.height as f32 / orig_h) * dh;
        let mut selection = el()
            .key("crop-selection")
            .absolute(left, top)
            .size(width.max(2.0), height.max(2.0))
            .border(1.5, t.amber)
            .shadow(0.0, 18.0, t.amber.alpha(0.28))
            .a11y_hidden();
        for (cx, cy) in [
            (0.0, 0.0),
            (width - 9.0, 0.0),
            (0.0, height - 9.0),
            (width - 9.0, height - 9.0),
        ] {
            selection = selection.child(
                el().absolute(cx - 4.0, cy - 4.0)
                    .size(9.0, 9.0)
                    .radius(2.0)
                    .fill(t.amber)
                    .border(1.0, rune::Color::BLACK.alpha(0.45))
                    .a11y_hidden(),
            );
        }
        for i in 1..3 {
            let f = i as f32 / 3.0;
            selection = selection
                .child(
                    el().absolute(width * f, 0.0)
                        .w(1.0)
                        .h(height)
                        .fill(rune::Color::WHITE.alpha(0.16))
                        .a11y_hidden(),
                )
                .child(
                    el().absolute(0.0, height * f)
                        .w(width)
                        .h(1.0)
                        .fill(rune::Color::WHITE.alpha(0.16))
                        .a11y_hidden(),
                );
        }
        stage = stage.child(selection);
    }

    let ratio = crop
        .map(|c| {
            if c.height > 0.0 {
                c.width / c.height
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);
    let numbers = crop
        .map(|c| {
            format!(
                "{:.0} x {:.0} px  ·  {:.2}:1  ·  at {:.0}, {:.0}",
                c.width, c.height, ratio, c.x, c.y
            )
        })
        .unwrap_or_else(|| {
            tr(
                state,
                "No original to crop",
                "No hay original para recortar",
            )
            .into()
        });

    let name = record
        .map(|r| {
            solar_rust::domain::catalog::channel(&r.channel)
                .map(|c| c.name.clone())
                .unwrap_or_else(|_| r.channel.clone())
        })
        .unwrap_or_else(|| "—".into());

    let header = row()
        .w_full()
        .center_y()
        .gap(10.0)
        .child(components::micro(
            tr(state, "Desktop crop", "Recorte de escritorio"),
            t.text_3,
        ))
        .child(el().w(1.0).h(12.0).fill(t.hairline).a11y_hidden())
        .child(components::label(name, theme::LABEL, t.text_2))
        .child(spacer().grow(1.0))
        .child(
            el().size(28.0, 28.0)
                .radius(8.0)
                .center()
                .hover(t.glass)
                .press(t.glass_strong)
                .focusable(true)
                .label("Close")
                .cursor(Cursor::Pointer)
                .on_press(|s: &mut State| s.cancel_crop())
                .child(rune::icon(&icons::CLOSE, 14.0).color(t.text_2)),
        );

    let footer = row()
        .w_full()
        .center_y()
        .gap(8.0)
        .child(components::label(numbers, theme::MONO_SM, t.text_3))
        .child(spacer().grow(1.0))
        .child(
            el().size(30.0, 30.0)
                .radius(9.0)
                .fill(t.glass_soft)
                .border(1.0, t.hairline)
                .center()
                .hover(t.glass)
                .focusable(true)
                .label("Zoom out")
                .cursor(Cursor::Pointer)
                .on_press(|s: &mut State| s.zoom_crop(0.88))
                .child(rune::icon(&icons::MINUS, 14.0).color(t.text_2)),
        )
        .child(
            el().size(30.0, 30.0)
                .radius(9.0)
                .fill(t.glass_soft)
                .border(1.0, t.hairline)
                .center()
                .hover(t.glass)
                .focusable(true)
                .label("Zoom in")
                .cursor(Cursor::Pointer)
                .on_press(|s: &mut State| s.zoom_crop(1.14))
                .child(rune::icon(&icons::PLUS, 14.0).color(t.text_2)),
        )
        .child(el().w(4.0).h(1.0).a11y_hidden())
        .child(ghost_button(
            tr(state, "Cancel", "Cancelar"),
            &icons::CLOSE,
            |s: &mut State| s.cancel_crop(),
        ))
        .child(accent_button(
            tr(state, "Save crop", "Guardar recorte"),
            &icons::CHECK,
            |s: &mut State| s.commit_crop(),
        ));

    el().key("crop-overlay")
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .fill(rune::Color::BLACK.alpha(0.58))
        .backdrop_blur(12.0)
        .center()
        .child(
            col()
                .key("crop-frame")
                .w(860.0)
                .h(548.0)
                .padding(18.0)
                .gap(12.0)
                .radius(20.0)
                .fill(t.panel)
                .border(1.0, t.hairline)
                .shadow(24.0, 60.0, rune::Color::BLACK.alpha(0.55))
                .child(header)
                .child(el().w_full().center().child(stage))
                .child(footer),
        )
}

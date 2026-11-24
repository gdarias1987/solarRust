//! Small, explicit Rune-FX integration for the two instrument surfaces that benefit from it.
//!
//! The compositor delegates ordinary drawing and isolated scopes to Rune's renderer. Its only
//! custom work is preparing a liquid-metal bezel and fluted-glass fill at tagged effect slots.
//! The info panel uses a translucent theme fill so glass samples the scene behind it; when
//! backdrop copies are unavailable, the compositor supplies a matte opaque fallback.

use rune::{
    compose::{Compositor, Frame},
    render::{EffectPhase, EffectSite, KIND_EFFECT},
};
use rune_fx::{
    FlutedGlassParams, Fx, FxFrame, FxLayer, FxOptions, FxRenderer, FxSlot, LiquidMetalParams,
    PlasticParams, Shape,
};

pub(super) const METAL: u32 = 0x534f_0001;
pub(super) const GLASS: u32 = 0x534f_0002;

#[derive(Default)]
pub(crate) struct SolarFx {
    renderer: Option<(rune::compose::DeviceKey, FxRenderer)>,
}

impl Compositor<super::State> for SolarFx {
    fn render(&mut self, frame: &mut Frame<'_, super::State>) {
        let device_key = frame.device_key();
        if self
            .renderer
            .as_ref()
            .is_none_or(|(key, _)| *key != device_key)
        {
            self.renderer = Some((
                device_key,
                FxRenderer::new(frame.device, frame.format, FxOptions::default()),
            ));
        }
        let fx = &mut self.renderer.as_mut().expect("renderer initialized").1;
        fx.set_srgb_blend(frame.srgb_blend());

        let mut layers = Vec::<FxLayer>::new();
        let mut slots = Vec::<FxSlot>::new();
        let mut marks = Vec::<(u32, usize, Option<usize>)>::new();
        for instance in &frame.rt.instances {
            if instance.params[1] != KIND_EFFECT {
                continue;
            }
            let mark =
                (instance.border[0] as u32 & 0xffff) | ((instance.border[1] as u32 & 0xffff) << 16);
            if mark != METAL && mark != GLASS {
                continue;
            }
            let scale = instance.params[3].max(0.001);
            let [x, y, w, h] = instance.rect.map(|v| v / scale);
            let radius = instance.radii.iter().copied().fold(0.0_f32, f32::max) / scale;
            let shape = Shape::rounded(x, y, w, h, radius);
            let layer = if mark == METAL {
                // A still chrome slab: the material remains real Rune-FX, but is deliberately
                // non-animated so the app returns to idle and reduced-motion needs no special
                // timer path.
                Fx::fill(
                    LiquidMetalParams {
                        flow: 0.0,
                        ..LiquidMetalParams::chrome()
                    }
                    .border(3.0)
                    .intensity(0.92),
                )
                .on(shape)
                .at(frame.rt.t as f32)
            } else {
                // Reduced refraction, dispersion and sheen keep the actual lensing visible
                // without striping small metadata; the muted tint calms the sampling backdrop.
                Fx::fill(FlutedGlassParams {
                    rib: 22.0,
                    refraction: 2.0,
                    dispersion: 0.01,
                    highlight: 0.08,
                    tint: rune_fx::rgb(0x7f8c96),
                    tint_amount: 0.24,
                    ..FlutedGlassParams::reeded()
                })
                .on(shape)
                .at(frame.rt.t as f32)
                .opacity(0.42)
            };
            let layer_index = layers.len();
            layers.push(layer);
            slots.push(
                FxSlot::default()
                    .scale(scale)
                    .clip(instance.clip, instance.fade),
            );
            let fallback_index = if mark == GLASS {
                // If the render target cannot be copied, replace the translucent UI backing
                // with a matte, opaque surface instead of leaving an unintended see-through pane.
                let fallback_index = layers.len();
                layers.push(
                    Fx::fill(PlasticParams {
                        color: rune_fx::rgb(0x121820),
                        roughness: 1.0,
                        dome: 0.0,
                        bevel: 0.0,
                        ..PlasticParams::matte()
                    })
                    .on(shape)
                    .at(frame.rt.t as f32),
                );
                slots.push(
                    FxSlot::default()
                        .scale(scale)
                        .clip(instance.clip, instance.fade),
                );
                Some(fallback_index)
            } else {
                None
            };
            marks.push((mark, layer_index, fallback_index));
        }

        frame.ui.prepare(
            frame.device,
            frame.queue,
            &mut frame.rt.text.atlas,
            &frame.rt.instances,
            frame.size,
        );
        let fx_frame = FxFrame {
            viewport: [frame.size.0, frame.size.1],
            scale: 1.0,
            layers: &layers,
            backdrop: None,
            sources: &[],
        };
        fx.prepare_with_slots(frame.device, frame.queue, &fx_frame, &slots);

        let clear = frame.rt.clear_color();
        frame.ui.draw_with_backdrop(
            frame.encoder,
            frame.target,
            clear,
            &mut |phase, site: &EffectSite<'_>| {
                let Some((_, layer_index, fallback_index)) =
                    marks.iter().find(|(mark, _, _)| *mark == site.mark)
                else {
                    return;
                };
                match phase {
                    EffectPhase::Backdrop(_) if site.mark == GLASS => {
                        if let Some(backdrop) = site.backdrop {
                            fx.set_layer_backdrop(frame.device, backdrop, *layer_index);
                        }
                    }
                    EffectPhase::Draw(pass) if site.mark == GLASS => {
                        if site.backdrop.is_some() {
                            fx.add_format(frame.device, site.format);
                            fx.draw_layer_in(pass, site.format, site.origin, *layer_index);
                        } else if let Some(index) = fallback_index {
                            fx.add_format(frame.device, site.format);
                            fx.draw_layer_in(pass, site.format, site.origin, *index);
                        }
                    }
                    EffectPhase::Draw(pass) => {
                        // Page transitions and opacity groups render into Rune's isolated
                        // Rgba16Float targets, not the window's Bgra8 surface. Select the
                        // pipeline and viewport for the actual effect site to avoid a wgpu
                        // attachment-format mismatch (and the resulting dropped/black frame).
                        fx.add_format(frame.device, site.format);
                        fx.draw_layer_in(pass, site.format, site.origin, *layer_index);
                    }
                    EffectPhase::Backdrop(_) => {}
                }
            },
        );
    }

    // Both fills are static; Rune can stop its display link when the UI itself is at rest.
    fn animating(&self) -> bool {
        false
    }
}

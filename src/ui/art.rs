//! Deterministic procedural solar imagery for the demo fixtures. The app never ships these
//! as data: they exist so a headless snapshot (and a first launch with no archive) shows the
//! observatory's visual language instead of an empty box.

pub struct Palette {
    pub core: [f32; 3],
    pub body: [f32; 3],
    pub corona: [f32; 3],
}

const fn p(core: [f32; 3], body: [f32; 3], corona: [f32; 3]) -> Palette {
    Palette { core, body, corona }
}

/// SDO-style false colour per product. Values are linear-ish RGB multipliers.
pub fn palette(id: &str) -> Palette {
    match id {
        "0094" => p([0.72, 1.0, 0.72], [0.10, 0.62, 0.30], [0.35, 1.0, 0.55]),
        "0131" => p([0.72, 1.0, 0.96], [0.05, 0.52, 0.58], [0.28, 1.0, 0.86]),
        "0171" => p([1.0, 0.93, 0.62], [0.86, 0.58, 0.10], [1.0, 0.78, 0.28]),
        "0193" => p([1.0, 0.90, 0.46], [0.88, 0.52, 0.06], [1.0, 0.72, 0.16]),
        "0211" => p([1.0, 0.62, 0.42], [0.72, 0.18, 0.09], [1.0, 0.40, 0.20]),
        "0304" => p([1.0, 0.70, 0.26], [1.0, 0.30, 0.04], [1.0, 0.46, 0.10]),
        "0335" => p([0.62, 0.82, 1.0], [0.12, 0.32, 0.82], [0.42, 0.72, 1.0]),
        "1600" => p([1.0, 0.97, 0.82], [0.86, 0.70, 0.42], [1.0, 0.88, 0.58]),
        "1700" => p([1.0, 0.84, 0.50], [0.74, 0.46, 0.14], [1.0, 0.72, 0.30]),
        "HMIB" | "HMIBC" | "HMIIC" | "HMIIF" | "HMII" | "HMID" => {
            p([0.92, 0.95, 1.0], [0.34, 0.40, 0.52], [0.55, 0.68, 0.92])
        }
        "COMP211193171" => p([1.0, 0.82, 0.55], [0.60, 0.38, 0.22], [0.95, 0.66, 0.34]),
        "COMP304211171" => p([1.0, 0.78, 0.42], [0.82, 0.34, 0.14], [1.0, 0.58, 0.24]),
        "COMP094335193" => p([0.82, 0.98, 0.78], [0.24, 0.52, 0.42], [0.55, 0.92, 0.70]),
        "COMPHMI171" => p([1.0, 0.90, 0.62], [0.58, 0.52, 0.36], [0.86, 0.76, 0.52]),
        other => {
            let h = other
                .bytes()
                .fold(7u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
            let palette = [
                p([1.0, 0.70, 0.26], [1.0, 0.30, 0.04], [1.0, 0.46, 0.10]),
                p([1.0, 0.93, 0.62], [0.86, 0.58, 0.10], [1.0, 0.78, 0.28]),
                p([0.62, 0.82, 1.0], [0.12, 0.32, 0.82], [0.42, 0.72, 1.0]),
                p([1.0, 0.62, 0.42], [0.72, 0.18, 0.09], [1.0, 0.40, 0.20]),
            ];
            Palette {
                core: palette[(h as usize) % palette.len()].core,
                body: palette[(h as usize) % palette.len()].body,
                corona: palette[(h as usize) % palette.len()].corona,
            }
        }
    }
}

/// The window icon: the observatory's Sun on a rounded slab, transparent outside the slab so
/// Windows draws a shape rather than a black square in the taskbar and the title bar. Encoded
/// as PNG for `rune::set_app_icon`, which decodes it at the sizes the shell shows (32 and 256).
pub fn app_icon_png(size: u32) -> Vec<u8> {
    use image::ImageEncoder;
    let rgba = app_icon_rgba(size);
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(&rgba, size, size, image::ExtendedColorType::Rgba8)
        .expect("in-memory PNG encoding cannot fail");
    bytes
}

/// A 2×2 supersampled icon sample: the slab's mask becomes the alpha, the artwork stays
/// opaque underneath so partially covered edge pixels do not fringe toward black.
fn app_icon_rgba(size: u32) -> Vec<u8> {
    let n = size as f32;
    let half = n * 0.5;
    let corner = n * 0.225;
    let inner = half - corner;
    let disc = n * 0.232;
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let mut rgb = [0.0f32; 3];
            let mut alpha = 0.0f32;
            for sy in 0..2 {
                for sx in 0..2 {
                    let px = x as f32 + 0.25 + sx as f32 * 0.5;
                    let py = y as f32 + 0.25 + sy as f32 * 0.5;
                    // Rounded-rect signed distance: the slab's coverage.
                    let qx = (px - half).abs() - inner;
                    let qy = (py - half).abs() - inner;
                    let outside = qx.max(0.0).hypot(qy.max(0.0));
                    let d = outside + qx.max(qy).min(0.0) - corner;

                    // Vertical slab gradient, navy to near-black, with two ambient lifts.
                    let t = (py / n * 0.65 + px / n * 0.35).clamp(0.0, 1.0);
                    let mut c = [
                        0.043 + (0.016 - 0.043) * t,
                        0.063 + (0.026 - 0.063) * t,
                        0.106 + (0.052 - 0.106) * t,
                    ];
                    let teal = (-(((px - n * 0.30).powi(2) + (py - n * 0.86).powi(2))
                        / (n * 0.42).powi(2)))
                    .exp();
                    let violet = (-(((px - n * 0.86).powi(2) + (py - n * 0.16).powi(2))
                        / (n * 0.38).powi(2)))
                    .exp();
                    c[0] += teal * 0.020 + violet * 0.055;
                    c[1] += teal * 0.105 + violet * 0.020;
                    c[2] += teal * 0.095 + violet * 0.120;

                    // The disc: granulation, limb darkening, and a white-hot rim.
                    let dx = px - half;
                    let dy = py - half;
                    let r = dx.hypot(dy);
                    let inside = (r / disc).min(1.0);
                    if r < disc {
                        let mu = (1.0 - inside * inside).max(0.0).sqrt();
                        let limb = 0.46 + 0.54 * mu.powf(0.62);
                        let grain =
                            (dx * 0.55 + dy * 0.28).sin() * (dy * 0.61 - dx * 0.19).sin() * 0.06;
                        let mix = inside.powf(1.15);
                        let body = [
                            1.0 * (1.0 - mix) + 1.0 * mix,
                            0.95 * (1.0 - mix) + 0.56 * mix,
                            0.78 * (1.0 - mix) + 0.12 * mix,
                        ];
                        let rim = (inside - 0.88).max(0.0) / 0.12;
                        for (channel, base) in c.iter_mut().zip(body) {
                            *channel =
                                base * (limb + grain) * (1.0 - rim * 0.25) + rim * rim * 0.30;
                        }
                    }
                    // Corona and rays, falling away from the limb.
                    if r >= disc * 0.94 {
                        let e = ((r - disc) / disc).max(0.0);
                        let glow = (-e * 5.2).exp() * 0.62 + (-e * 1.7).exp() * 0.10;
                        let angle = dy.atan2(dx);
                        let rays =
                            (angle * 8.0 + 0.7).sin().max(0.0).powi(6) * (-e * 3.4).exp() * 0.26;
                        let k = glow + rays;
                        c[0] += k * 1.0;
                        c[1] += k * 0.62;
                        c[2] += k * 0.28;
                    }
                    // A hairline accent just inside the slab's edge.
                    if (-2.0..-0.4).contains(&d) {
                        let rim = ((-d - 0.4) / 1.6).clamp(0.0, 1.0);
                        c[0] += rim * 0.055;
                        c[1] += rim * 0.190;
                        c[2] += rim * 0.175;
                    }

                    rgb[0] += c[0];
                    rgb[1] += c[1];
                    rgb[2] += c[2];
                    alpha += if d <= 0.0 { 1.0 } else { 0.0 };
                }
            }
            let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            out.extend_from_slice(&[
                to_u8(rgb[0] / 4.0),
                to_u8(rgb[1] / 4.0),
                to_u8(rgb[2] / 4.0),
                to_u8(alpha / 4.0),
            ]);
        }
    }
    out
}

fn hash(mut x: u32) -> f32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    (x as f32) / (u32::MAX as f32)
}

/// A square RGBA frame: photosphere with limb darkening, granulation, active regions and a
/// corona. `seed` shifts the active regions and ray phase so fixtures differ.
pub fn sun_rgba(size: u32, seed: u32, palette: &Palette) -> Vec<u8> {
    let n = size as f32;
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    let disc = 0.78;
    for y in 0..size {
        for x in 0..size {
            let nx = (x as f32 + 0.5) / n * 2.0 - 1.0;
            let ny = (y as f32 + 0.5) / n * 2.0 - 1.0;
            let r = (nx * nx + ny * ny).sqrt();
            let angle = ny.atan2(nx);

            // Deep space with a faint blue-violet lift, never pure black.
            let mut rgb = [0.006, 0.008, 0.014];

            if r < disc + 0.03 {
                let inside = (r / disc).min(1.0);
                let mu = (1.0 - inside * inside).max(0.0).sqrt();
                let limb = 0.26 + 0.74 * mu.powf(0.55);

                // Granulation: a few irrational frequencies so it never tiles visibly.
                let g = (nx * 26.0 + ny * 9.0 + seed as f32).sin() * 0.45
                    + (ny * 41.0 - nx * 17.0).sin() * 0.32
                    + (nx * 73.0 + ny * 59.0).sin() * 0.16;

                // Three active regions whose positions move with the seed.
                let s = seed as f32 * 0.7;
                let mut spots = 0.0;
                for i in 0..3 {
                    let a = s + i as f32 * 2.31;
                    let d = 0.26 + i as f32 * 0.11;
                    let cx = a.cos() * d;
                    let cy = (a * 0.8).sin() * d * 0.7;
                    let dist2 = (nx - cx) * (nx - cx) + (ny - cy) * (ny - cy);
                    spots += (-dist2 * 70.0).exp() * 0.55;
                }

                let bright = (limb * (1.0 + 0.07 * g) + spots).max(0.0);
                let heat = (spots * 1.4).min(1.0);
                for (channel, (body, core)) in rgb
                    .iter_mut()
                    .zip(palette.body.iter().zip(palette.core.iter()))
                {
                    *channel = *body * bright + *core * heat * 0.85;
                }
            }

            // Corona and rays outside the limb.
            if r > disc - 0.04 {
                let edge = (r - disc).max(0.0);
                let glow = (-edge * 7.5).exp() * 0.55 + (-edge * 2.2).exp() * 0.10;
                let rays = (angle * (9.0 + (seed % 5) as f32) + seed as f32 * 0.3)
                    .sin()
                    .max(0.0)
                    .powi(3);
                let ray = rays * (-edge * 3.4).exp() * 0.16;
                for (channel, corona) in rgb.iter_mut().zip(palette.corona.iter()) {
                    *channel += *corona * (glow + ray);
                }
            }

            // A little dither so large gradients do not band after 8-bit encoding.
            let dither = (hash(x.wrapping_mul(18_081).wrapping_add(y) ^ seed) - 0.5) * 0.012;
            let to_u8 = |v: f32| ((v + dither).clamp(0.0, 1.0) * 255.0).round() as u8;
            out.extend_from_slice(&[to_u8(rgb[0]), to_u8(rgb[1]), to_u8(rgb[2]), 255]);
        }
    }
    out
}

/// The executable's icon resource: the same artwork at every size Windows asks the shell for,
/// packed as an `.ico` (PNG payloads, which Vista and later read at any size). `build.rs` embeds
/// the committed `assets/app.ico`, so Explorer, Start, pinning and the taskbar's group icon agree
/// with the running window. Test-only because it exists to regenerate that file; the binary
/// itself only needs `app_icon_png`.
#[cfg(test)]
pub fn app_icon_ico() -> Vec<u8> {
    const SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];
    let images: Vec<Vec<u8>> = SIZES.iter().map(|&size| app_icon_png(size)).collect();
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes()); // reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // 1 = icon
    out.extend_from_slice(&(SIZES.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * SIZES.len() as u32;
    for (&size, png) in SIZES.iter().zip(&images) {
        // 0 means 256 in an ICO directory entry.
        out.push(if size == 256 { 0 } else { size as u8 });
        out.push(if size == 256 { 0 } else { size as u8 });
        out.push(0); // palette size
        out.push(0); // reserved
        out.extend_from_slice(&1u16.to_le_bytes()); // planes
        out.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
        out.extend_from_slice(&(png.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for png in images {
        out.extend_from_slice(&png);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_is_a_transparent_cornered_square() {
        let png = app_icon_png(64);
        // Set SOLAR_ICON_DUMP to a path to look at the generated art.
        if let Ok(path) = std::env::var("SOLAR_ICON_DUMP") {
            std::fs::write(path, &png).expect("icon dump is writable");
        }
        let icon = image::load_from_memory(&png)
            .expect("icon is a PNG")
            .to_rgba8();
        assert_eq!(icon.dimensions(), (64, 64));
        assert_eq!(icon.get_pixel(0, 0)[3], 0, "the slab's corners stay clear");
        assert_eq!(icon.get_pixel(32, 32)[3], 255, "the Sun covers the middle");

        // The same artwork has to hold up wherever Windows scales it: the taskbar's 24 px, the
        // title bar's 16, the 256 Explorer asks for. Alpha is what keeps the slab a shape.
        for size in [16u32, 24, 32, 48, 64, 128, 256] {
            let img = image::load_from_memory(&app_icon_png(size))
                .expect("icon is a PNG")
                .to_rgba8();
            let half = size / 2;
            assert_eq!(
                img.get_pixel(half, half)[3],
                255,
                "the Sun covers the middle at {size}"
            );
            assert_eq!(
                img.get_pixel(0, 0)[3],
                0,
                "the corners stay clear at {size}"
            );
            assert_eq!(
                img.get_pixel(half, 0)[3],
                255,
                "the slab reaches the edge at {size}"
            );
        }
    }

    #[test]
    fn icon_resource_is_an_ico_with_every_size() {
        let ico = app_icon_ico();
        assert_eq!(&ico[0..4], &[0, 0, 1, 0], "an ICO header");
        let count = u16::from_le_bytes([ico[4], ico[5]]) as usize;
        assert_eq!(count, 7, "16 through 256");
        for entry in 0..count {
            let at = 6 + entry * 16;
            let width = if ico[at] == 0 { 256 } else { ico[at] as u32 };
            let height = if ico[at + 1] == 0 {
                256
            } else {
                ico[at + 1] as u32
            };
            assert_eq!(width, height, "square entries");
            let len = u32::from_le_bytes([ico[at + 8], ico[at + 9], ico[at + 10], ico[at + 11]]);
            let off = u32::from_le_bytes([ico[at + 12], ico[at + 13], ico[at + 14], ico[at + 15]]);
            let payload = &ico[off as usize..(off + len) as usize];
            assert_eq!(&payload[1..4], b"PNG", "PNG payloads at {width} px");
            let decoded = image::load_from_memory(payload).expect("payload is a PNG");
            assert_eq!(
                decoded.width(),
                width,
                "payload matches its directory entry"
            );
        }
        // `assets/app.ico` is the file `build.rs` embeds. Set SOLAR_ICON_WRITE=1 to regenerate it
        // after changing the art; otherwise the committed copy must match byte for byte, so a
        // change to the drawing cannot leave the executable showing the old Sun.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/app.ico");
        if std::env::var("SOLAR_ICON_WRITE").is_ok() {
            std::fs::write(&path, &ico).expect("icon resource is writable");
        } else {
            let committed = std::fs::read(&path).expect("assets/app.ico is committed");
            assert_eq!(
                committed, ico,
                "regenerate assets/app.ico with SOLAR_ICON_WRITE=1"
            );
        }
    }
}

//! Static reference images copied from solar_go; these are not live observations.
use rune::Pixels;
use std::collections::HashMap;

const IMAGES: &[(&str, &[u8])] = &[
    (
        "0094",
        include_bytes!("../../content/previews/latest_256_0094.jpg"),
    ),
    (
        "0131",
        include_bytes!("../../content/previews/latest_256_0131.jpg"),
    ),
    (
        "0171",
        include_bytes!("../../content/previews/latest_256_0171.jpg"),
    ),
    (
        "0193",
        include_bytes!("../../content/previews/latest_256_0193.jpg"),
    ),
    (
        "0211",
        include_bytes!("../../content/previews/latest_256_0211.jpg"),
    ),
    (
        "0304",
        include_bytes!("../../content/previews/latest_256_0304.jpg"),
    ),
    (
        "0335",
        include_bytes!("../../content/previews/latest_256_0335.jpg"),
    ),
    (
        "1600",
        include_bytes!("../../content/previews/latest_256_1600.jpg"),
    ),
    (
        "1700",
        include_bytes!("../../content/previews/latest_256_1700.jpg"),
    ),
    (
        "HMIB",
        include_bytes!("../../content/previews/latest_256_HMIB.jpg"),
    ),
    (
        "HMIBC",
        include_bytes!("../../content/previews/latest_256_HMIBC.jpg"),
    ),
    (
        "HMIIC",
        include_bytes!("../../content/previews/latest_256_HMIIC.jpg"),
    ),
    (
        "HMIIF",
        include_bytes!("../../content/previews/latest_256_HMIIF.jpg"),
    ),
    (
        "HMII",
        include_bytes!("../../content/previews/latest_256_HMII.jpg"),
    ),
    (
        "HMID",
        include_bytes!("../../content/previews/latest_256_HMID.jpg"),
    ),
    (
        "COMP211193171",
        include_bytes!("../../content/previews/latest_256_211193171.jpg"),
    ),
    (
        "COMP304211171",
        include_bytes!("../../content/previews/f_304_211_171_256.jpg"),
    ),
    (
        "COMP094335193",
        include_bytes!("../../content/previews/f_094_335_193_256.jpg"),
    ),
    (
        "COMPHMI171",
        include_bytes!("../../content/previews/f_HMImag_171_256.jpg"),
    ),
];

pub fn load() -> HashMap<String, Pixels> {
    IMAGES
        .iter()
        .filter_map(|(id, data)| {
            let image = image::load_from_memory(data).ok()?.to_rgba8();
            let pixels = Pixels::new();
            pixels.set(image.width(), image.height(), image.into_raw());
            Some(((*id).to_string(), pixels))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_catalog_channels_have_valid_reference_images() {
        let images = super::load();
        for channel in solar_rust::domain::catalog::channels() {
            assert!(
                images.contains_key(&channel.id),
                "missing reference image for {}",
                channel.id
            );
        }
        assert_eq!(images.len(), solar_rust::domain::catalog::channels().len());
    }
}

//! The app's icon set: 16 pt inline SVGs, stroked with `currentColor` so `El::icon`
//! tints each one with the element's colour.
//!
//! A curated catalogue: pages draw the subset they need, and the rest stay here as the
//! vocabulary for the next surface.
#![allow(dead_code)]

use rune::Icon;

pub static SUN: Icon = Icon {
    name: "solar-sun",
    view: 16.0,
    body: r#"<circle cx="8" cy="8" r="3.1" stroke="currentColor" stroke-width="1.2"/><path d="M8 1.5V2.9M8 13.1V14.5M1.5 8H2.9M13.1 8H14.5M3.4 3.4L4.4 4.4M11.6 11.6L12.6 12.6M12.6 3.4L11.6 4.4M4.4 11.6L3.4 12.6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>"#,
};

pub static LAYERS: Icon = Icon {
    name: "solar-layers",
    view: 16.0,
    body: r#"<path d="M8 1.8L14.2 5L8 8.2L1.8 5L8 1.8Z" stroke="currentColor" stroke-width="1.15" stroke-linejoin="round"/><path d="M2.4 8L8 11L13.6 8M2.4 10.9L8 13.9L13.6 10.9" stroke="currentColor" stroke-width="1.15" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static GALLERY: Icon = Icon {
    name: "solar-gallery",
    view: 16.0,
    body: r#"<rect x="1.8" y="2.2" width="12.4" height="11.6" rx="2.2" stroke="currentColor" stroke-width="1.2"/><circle cx="5.5" cy="6" r="1.15" fill="currentColor"/><path d="M2.5 12.3L6.2 8.7L8.6 10.7L10.6 8.9L13.5 11.7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static SLIDERS: Icon = Icon {
    name: "solar-sliders",
    view: 16.0,
    body: r#"<path d="M2 4.4H14M2 8H14M2 11.6H14" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"/><circle cx="6" cy="4.4" r="1.7" fill="currentColor"/><circle cx="10.4" cy="8" r="1.7" fill="currentColor"/><circle cx="5" cy="11.6" r="1.7" fill="currentColor"/>"#,
};

pub static CHART: Icon = Icon {
    name: "solar-chart",
    view: 16.0,
    body: r#"<path d="M2.3 13.5V9M6 13.5V5.2M9.7 13.5V7.3M13.7 13.5V2.6" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"/>"#,
};

pub static INFO: Icon = Icon {
    name: "solar-info",
    view: 16.0,
    body: r#"<circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.2"/><path d="M8 7.2V11.3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/><circle cx="8" cy="4.9" r="0.85" fill="currentColor"/>"#,
};

pub static SYNC: Icon = Icon {
    name: "solar-sync",
    view: 16.0,
    body: r#"<path d="M13.2 8.3A5.2 5.2 0 1 1 11.5 4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/><path d="M12.7 1.5L12.1 4.7L9 4.1" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static APPLY: Icon = Icon {
    name: "solar-apply",
    view: 16.0,
    body: r#"<rect x="1.9" y="2.4" width="12.2" height="8.3" rx="1.6" stroke="currentColor" stroke-width="1.2"/><path d="M5.7 13.7H10.3M8 10.9V13.7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>"#,
};

pub static CROP: Icon = Icon {
    name: "solar-crop",
    view: 16.0,
    body: r#"<path d="M4.2 1.7V11.8H14.3M1.7 4.2H11.8V14.3" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static FOLDER: Icon = Icon {
    name: "solar-folder",
    view: 16.0,
    body: r#"<path d="M1.8 4.7C1.8 3.8 2.5 3.1 3.4 3.1H5.9L7.4 4.9H12.6C13.5 4.9 14.2 5.6 14.2 6.5V12.1C14.2 13 13.5 13.7 12.6 13.7H3.4C2.5 13.7 1.8 13 1.8 12.1V4.7Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/>"#,
};

pub static CHECK: Icon = Icon {
    name: "solar-check",
    view: 16.0,
    body: r#"<path d="M3.2 8.4L6.4 11.5L12.8 4.6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static CLOSE: Icon = Icon {
    name: "solar-close",
    view: 16.0,
    body: r#"<path d="M4.2 4.2L11.8 11.8M11.8 4.2L4.2 11.8" stroke="currentColor" stroke-width="1.35" stroke-linecap="round"/>"#,
};

pub static CHEVRON_RIGHT: Icon = Icon {
    name: "solar-chevron-right",
    view: 16.0,
    body: r#"<path d="M6.2 3.6L10.6 8L6.2 12.4" stroke="currentColor" stroke-width="1.35" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static ARROW_LEFT: Icon = Icon {
    name: "solar-arrow-left",
    view: 16.0,
    body: r#"<path d="M13 8H3M3 8L6.6 4.4M3 8L6.6 11.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static ARROW_RIGHT: Icon = Icon {
    name: "solar-arrow-right",
    view: 16.0,
    body: r#"<path d="M3 8H13M13 8L9.4 4.4M13 8L9.4 11.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static DOWNLOAD: Icon = Icon {
    name: "solar-download",
    view: 16.0,
    body: r#"<path d="M8 2.1V9.9M8 9.9L4.8 6.7M8 9.9L11.2 6.7" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/><path d="M2.6 11.8V12.6C2.6 13.3 3.2 13.9 3.9 13.9H12.1C12.8 13.9 13.4 13.3 13.4 12.6V11.8" stroke="currentColor" stroke-width="1.25" stroke-linecap="round"/>"#,
};

pub static CLOCK: Icon = Icon {
    name: "solar-clock",
    view: 16.0,
    body: r#"<circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.2"/><path d="M8 4.5V8.2L10.7 9.9" stroke="currentColor" stroke-width="1.25" stroke-linecap="round"/>"#,
};

pub static DISK: Icon = Icon {
    name: "solar-disk",
    view: 16.0,
    body: r#"<ellipse cx="8" cy="3.7" rx="5.4" ry="2" stroke="currentColor" stroke-width="1.15"/><path d="M2.6 3.7V12.3C2.6 13.4 5 14.3 8 14.3C11 14.3 13.4 13.4 13.4 12.3V3.7" stroke="currentColor" stroke-width="1.15"/><path d="M2.6 8C2.6 9.1 5 10 8 10C11 10 13.4 9.1 13.4 8" stroke="currentColor" stroke-width="1.15"/>"#,
};

pub static GLOBE: Icon = Icon {
    name: "solar-globe",
    view: 16.0,
    body: r#"<circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.15"/><path d="M2 8H14M8 2C9.7 3.6 10.7 5.7 10.7 8C10.7 10.3 9.7 12.4 8 14C6.3 12.4 5.3 10.3 5.3 8C5.3 5.7 6.3 3.6 8 2Z" stroke="currentColor" stroke-width="1.05"/>"#,
};

pub static SPARKLE: Icon = Icon {
    name: "solar-sparkle",
    view: 16.0,
    body: r#"<path d="M8 1.6L9.35 6.05L13.8 7.4L9.35 8.75L8 13.2L6.65 8.75L2.2 7.4L6.65 6.05L8 1.6Z" fill="currentColor"/>"#,
};

pub static EXPAND: Icon = Icon {
    name: "solar-expand",
    view: 16.0,
    body: r#"<path d="M6.1 2.2H2.2V6.1M9.9 2.2H13.8V6.1M13.8 9.9V13.8H9.9M6.1 13.8H2.2V9.9" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static TARGET: Icon = Icon {
    name: "solar-target",
    view: 16.0,
    body: r#"<circle cx="8" cy="8" r="5.2" stroke="currentColor" stroke-width="1.2"/><circle cx="8" cy="8" r="1.5" fill="currentColor"/><path d="M8 1.4V3M8 13V14.6M1.4 8H3M13 8H14.6" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"/>"#,
};

pub static IMAGE: Icon = Icon {
    name: "solar-image",
    view: 16.0,
    body: r#"<rect x="1.9" y="2.6" width="12.2" height="10.8" rx="2" stroke="currentColor" stroke-width="1.2"/><circle cx="5.4" cy="6.2" r="1.1" fill="currentColor"/><path d="M2.4 12.4L6 8.9L8.2 10.7L10.2 9.1L13.6 12.1" stroke="currentColor" stroke-width="1.15" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static WARN: Icon = Icon {
    name: "solar-warn",
    view: 16.0,
    body: r#"<path d="M8 2.2L14.4 13.4H1.6L8 2.2Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/><path d="M8 6.2V9.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/><circle cx="8" cy="11.7" r="0.8" fill="currentColor"/>"#,
};

pub static PLUS: Icon = Icon {
    name: "solar-plus",
    view: 16.0,
    body: r#"<path d="M8 3V13M3 8H13" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>"#,
};

pub static MINUS: Icon = Icon {
    name: "solar-minus",
    view: 16.0,
    body: r#"<path d="M3 8H13" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>"#,
};

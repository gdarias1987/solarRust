pub mod catalog;
pub mod crop;
pub mod lifecycle;

use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum Language {
    #[default]
    En,
    Es,
}
impl Language {
    pub fn detect(locale: &str) -> Self {
        if locale.to_ascii_lowercase().starts_with("es") {
            Self::Es
        } else {
            Self::En
        }
    }
    pub fn text<'a>(self, en: &'a str, es: &'a str) -> &'a str {
        match self {
            Self::En => en,
            Self::Es => es,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Fit,
    Crop,
    Center,
    Span,
    SolarCrop,
}
impl Mode {
    pub const ALL: [Self; 5] = [
        Self::Fit,
        Self::Crop,
        Self::Center,
        Self::Span,
        Self::SolarCrop,
    ];
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Preferences {
    pub channel: String,
    pub language: Language,
    pub mode: Mode,
    pub crop: Option<crop::Crop>,
    pub interval_seconds: u64,
    pub retention: usize,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            channel: "0304".into(),
            language: Language::detect(&sys_locale::get_locale().unwrap_or_default()),
            mode: Mode::Fit,
            crop: None,
            interval_seconds: 300,
            retention: 10,
        }
    }
}
impl Preferences {
    pub fn validate(&self) -> Result<()> {
        catalog::channel(&self.channel)?;
        if self.interval_seconds < 10 || self.retention == 0 {
            return Err(Error::Invalid(
                "interval >= 10 seconds and retention >= 1 required".into(),
            ));
        }
        if let Some(crop) = &self.crop {
            crop.validate()?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Record {
    pub id: String,
    pub channel: String,
    pub url: String,
    pub remote_modified: Option<String>,
    pub downloaded_at: u64,
    pub bytes: u64,
    pub file: String,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Archive {
    pub preferences: Preferences,
    pub records: Vec<Record>,
}
impl Archive {
    pub fn latest(&self, channel: &str) -> Option<&Record> {
        self.records
            .iter()
            .filter(|r| r.channel == channel)
            .max_by_key(|r| r.downloaded_at)
    }
    pub fn statistics(&self, channel: Option<&str>) -> (usize, u64, Option<u64>) {
        self.records
            .iter()
            .filter(|r| channel.is_none_or(|c| r.channel == c))
            .fold((0, 0, None), |(n, b, t), r| {
                (
                    n + 1,
                    b + r.bytes,
                    Some(t.unwrap_or(0).max(r.downloaded_at)),
                )
            })
    }
}

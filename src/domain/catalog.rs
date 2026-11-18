use super::Language;
use crate::{Error, Result};
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub wavelength: String,
    pub ions: String,
    pub region: String,
    pub kelvin_min: f64,
    pub kelvin_max: f64,
    pub url: String,
    /// Spanish copy, kept beside the English source so the catalog stays one file.
    #[serde(default)]
    pub description_es: String,
    #[serde(default)]
    pub region_es: String,
    /// The same channel for two more readers: no jargon, and a child's picture of it.
    #[serde(default)]
    pub simple: String,
    #[serde(default)]
    pub simple_es: String,
    #[serde(default)]
    pub kids: String,
    #[serde(default)]
    pub kids_es: String,
}
impl Channel {
    /// Channel prose in the interface language, falling back to English when untranslated.
    pub fn description(&self, language: Language) -> &str {
        match language {
            Language::Es if !self.description_es.is_empty() => &self.description_es,
            _ => &self.description,
        }
    }
    pub fn region(&self, language: Language) -> &str {
        match language {
            Language::Es if !self.region_es.is_empty() => &self.region_es,
            _ => &self.region,
        }
    }
    /// The channel without jargon, for a reader who is not an astronomer.
    pub fn simple(&self, language: Language) -> &str {
        match language {
            Language::Es if !self.simple_es.is_empty() => &self.simple_es,
            _ => &self.simple,
        }
    }
    /// The channel as a picture a child can hold on to.
    pub fn kids(&self, language: Language) -> &str {
        match language {
            Language::Es if !self.kids_es.is_empty() => &self.kids_es,
            _ => &self.kids,
        }
    }
}
pub fn channels() -> &'static [Channel] {
    static CHANNELS: OnceLock<Vec<Channel>> = OnceLock::new();
    CHANNELS.get_or_init(|| {
        serde_json::from_str(include_str!("../../content/channels.json"))
            .expect("bundled catalog tested at build time")
    })
}
pub fn channel(id: &str) -> Result<&'static Channel> {
    channels()
        .iter()
        .find(|c| c.id == id)
        .ok_or_else(|| Error::Invalid(format!("Unknown channel {id}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_parses_and_is_translated() {
        let all = channels();
        assert_eq!(all.len(), 19, "the catalog ships nineteen instruments");
        for channel in all {
            assert!(!channel.url.is_empty(), "{} needs a source url", channel.id);
            assert!(
                !channel.description_es.is_empty() && !channel.region_es.is_empty(),
                "{} is missing Spanish copy",
                channel.id
            );
            assert!(
                !channel.simple.is_empty()
                    && !channel.simple_es.is_empty()
                    && !channel.kids.is_empty()
                    && !channel.kids_es.is_empty(),
                "{} is missing a plain-language or child explanation",
                channel.id
            );
            assert!(channel.kelvin_max >= channel.kelvin_min);
        }
    }

    #[test]
    fn unknown_channel_is_an_error() {
        assert!(channel("nope").is_err());
    }
}

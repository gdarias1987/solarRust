use crate::{Error, Result};
use serde::{Deserialize, Serialize};

/// Coordinates are original-image pixels, never preview pixels.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Crop {
    pub source: String,
    pub original_width: u32,
    pub original_height: u32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl Crop {
    pub fn validate(&self) -> Result<()> {
        if self.source.is_empty()
            || self.original_width == 0
            || self.original_height == 0
            || ![self.x, self.y, self.width, self.height]
                .iter()
                .all(|x| x.is_finite())
            || self.x < 0.0
            || self.y < 0.0
            || self.width < 1.0
            || self.height < 1.0
            || self.x + self.width > self.original_width as f64 + 1e-6
            || self.y + self.height > self.original_height as f64 + 1e-6
        {
            return Err(Error::Invalid(
                "Crop must be finite, positive and inside the original image".into(),
            ));
        }
        Ok(())
    }
    pub fn centered(source: String, width: u32, height: u32, aspect: f64) -> Result<Self> {
        if !aspect.is_finite() || aspect <= 0.0 {
            return Err(Error::Invalid("Invalid aspect ratio".into()));
        }
        let w = (width as f64).min(height as f64 * aspect);
        let h = w / aspect;
        let crop = Self {
            source,
            original_width: width,
            original_height: height,
            x: (width as f64 - w) / 2.0,
            y: (height as f64 - h) / 2.0,
            width: w,
            height: h,
        };
        crop.validate()?;
        Ok(crop)
    }
    pub fn scaled(&self, width: u32, height: u32) -> Result<(u32, u32, u32, u32)> {
        self.validate()?;
        if width == 0 || height == 0 {
            return Err(Error::Invalid("Empty target image".into()));
        }
        let x =
            ((self.x / self.original_width as f64 * width as f64).floor() as u32).min(width - 1);
        let y =
            ((self.y / self.original_height as f64 * height as f64).floor() as u32).min(height - 1);
        let w = ((self.width / self.original_width as f64 * width as f64).round() as u32)
            .clamp(1, width - x);
        let h = ((self.height / self.original_height as f64 * height as f64).round() as u32)
            .clamp(1, height - y);
        Ok((x, y, w, h))
    }
    pub fn adjust(&mut self, dx: f64, dy: f64, scale: f64) {
        let w = (self.width * scale).clamp(1.0, self.original_width as f64);
        let h = w / (self.width / self.height);
        if h > self.original_height as f64 || h < 1.0 {
            return;
        }
        self.x = (self.x + dx + (self.width - w) / 2.0).clamp(0.0, self.original_width as f64 - w);
        self.y =
            (self.y + dy + (self.height - h) / 2.0).clamp(0.0, self.original_height as f64 - h);
        self.width = w;
        self.height = h;
    }
}

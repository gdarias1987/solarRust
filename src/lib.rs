//! Solar Observatory: domain rules and serialized application operations.
pub mod application;
pub mod domain;
pub mod infrastructure;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid data: {0}")]
    Invalid(String),
    #[error("Storage: {0}")]
    Storage(#[from] std::io::Error),
    #[error("Metadata: {0}")]
    Metadata(#[from] serde_json::Error),
    #[error("Network: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Image: {0}")]
    Image(#[from] image::ImageError),
    #[error("Wallpaper adapter is not implemented on this platform")]
    UnsupportedWallpaper,
}
pub type Result<T> = std::result::Result<T, Error>;

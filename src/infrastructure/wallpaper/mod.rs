//! Putting an image on the desktop is the one job no two systems agree on, so every backend
//! sits behind the same two calls: [`apply`] shows an image, [`active`] reports what is on
//! the desktop now.
//!
//! `active` answers `None` when the answer is unknown, and the retention rules read that as
//! "do not delete anything" — so a backend that cannot read the desktop back says so instead
//! of guessing. Where the system offers a read-back, the backend verifies its own write.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
// The Linux plan is plain data: detection reads the environment and the plan is a list of
// commands, so it compiles on every host and its tests run everywhere. Only the dispatcher
// below is platform-gated.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod linux;

#[cfg(windows)]
pub use windows::{active, apply};

#[cfg(target_os = "macos")]
pub use macos::{active, apply};

#[cfg(target_os = "linux")]
pub use linux::{active, apply};

/// No adapter exists for this platform, and saying so is better than pretending the image
/// was applied.
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn apply(_path: &std::path::Path, _mode: crate::domain::Mode) -> crate::Result<()> {
    Err(crate::Error::UnsupportedWallpaper)
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn active() -> Option<std::path::PathBuf> {
    None
}

/// `mode` is a request about how the picture meets the screen, not about a specific desktop;
/// the names below are the closest thing to a common vocabulary the desktops share.
#[cfg(any(target_os = "macos", target_os = "linux", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Placement {
    /// The whole picture, letterboxed when the aspect ratio differs.
    Fit,
    /// The picture fills the screen, cropping whatever hangs over the edge.
    Fill,
    /// Natural size, centered, with the desktop showing around it.
    Center,
}

#[cfg(any(target_os = "macos", target_os = "linux", test))]
pub(crate) fn placement(mode: crate::domain::Mode) -> Placement {
    use crate::domain::Mode;
    match mode {
        Mode::Fit => Placement::Fit,
        Mode::Center => Placement::Center,
        // `Span` has no cross-screen equivalent outside Windows, and a solar crop is already
        // cut to the screen, so both fill the screen the way `Crop` does.
        Mode::Crop | Mode::Span | Mode::SolarCrop => Placement::Fill,
    }
}

/// An AppleScript string literal. The path is ours in practice, but `apply` is public and a
/// quote or a backslash would otherwise break the script instead of failing cleanly.
#[cfg(any(target_os = "macos", test))]
pub(crate) fn applescript_literal(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        if c == '"' || c == '\\' {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Mode;

    #[test]
    fn modes_map_to_how_the_picture_meets_the_screen() {
        assert_eq!(placement(Mode::Fit), Placement::Fit);
        assert_eq!(placement(Mode::Center), Placement::Center);
        assert_eq!(placement(Mode::Crop), Placement::Fill);
        assert_eq!(placement(Mode::Span), Placement::Fill);
        assert_eq!(placement(Mode::SolarCrop), Placement::Fill);
    }

    #[test]
    fn applescript_literals_survive_quotes_and_backslashes() {
        assert_eq!(applescript_literal("/a/b.jpg"), r#""/a/b.jpg""#);
        assert_eq!(
            applescript_literal(r#"/a/we"ird\b.jpg"#),
            r#""/a/we\"ird\\b.jpg""#
        );
    }
}

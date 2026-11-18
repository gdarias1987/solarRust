//! Linux has no desktop API: each environment publishes its own tool and its own names for
//! the same three ideas. So detection is a string lookup, the work is a list of commands, and
//! both are plain data — which is also why this file compiles and is tested on every host.
//!
//! Read-back only exists where a settings daemon owns the picture (GNOME family and XFCE).
//! Elsewhere [`active`] answers `None`, which the retention rules already read as "unknown".

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{Error, Result, domain::Mode};

const GNOME: &str = "org.gnome.desktop.background";
const CINNAMON: &str = "org.cinnamon.desktop.background";
const MATE: &str = "org.mate.background";
const XFCE: &str = "xfce4-desktop";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Desktop {
    Kde,
    Cinnamon,
    Mate,
    Xfce,
    Gnome,
    Hyprland,
    Sway,
    /// No compositor recognised: whichever X11 tool happens to be installed.
    X11,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Step {
    program: &'static str,
    args: Vec<String>,
    /// A setting the desktop may not have, whose absence must not fail the apply.
    optional: bool,
}

fn step(program: &'static str, args: &[&str]) -> Step {
    Step {
        program,
        args: args.iter().map(|arg| (*arg).to_string()).collect(),
        optional: false,
    }
}

fn soft(program: &'static str, args: &[&str]) -> Step {
    Step {
        optional: true,
        ..step(program, args)
    }
}

/// `read` answers environment variables; keeping it a parameter is what lets the detection
/// order be tested without owning the machine.
fn detect(read: &dyn Fn(&str) -> Option<String>) -> Option<Desktop> {
    let names: Vec<String> = [
        "XDG_CURRENT_DESKTOP",
        "XDG_SESSION_DESKTOP",
        "DESKTOP_SESSION",
    ]
    .iter()
    .filter_map(|key| read(key))
    .map(|value| value.to_ascii_lowercase())
    .collect();
    let named = |needle: &str| {
        names
            .iter()
            .any(|name| name.split(':').any(|part| part == needle))
    };

    // Order matters: Plasma and Cinnamon also set `XDG_CURRENT_DESKTOP` to a value that
    // contains their own name, and a nested session reports the outer one first.
    if named("kde") || named("plasma") {
        return Some(Desktop::Kde);
    }
    if named("cinnamon") || named("x-cinnamon") {
        return Some(Desktop::Cinnamon);
    }
    if named("mate") {
        return Some(Desktop::Mate);
    }
    if named("xfce") {
        return Some(Desktop::Xfce);
    }
    if named("gnome") || named("ubuntu") || named("unity") || named("pantheon") {
        return Some(Desktop::Gnome);
    }
    if named("hyprland") || read("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        return Some(Desktop::Hyprland);
    }
    if named("sway") || read("SWAYSOCK").is_some() {
        return Some(Desktop::Sway);
    }
    if read("DISPLAY").is_some() || read("WAYLAND_DISPLAY").is_some() {
        return Some(Desktop::X11);
    }
    None
}

/// Paths reach `gsettings` as URLs, where everything outside the unreserved set has to be
/// escaped — a space in a directory name is otherwise a broken setting.
fn file_uri(path: &Path) -> String {
    let mut uri = String::from("file://");
    for &byte in path.to_string_lossy().as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                uri.push(byte as char)
            }
            _ => uri.push_str(&format!("%{byte:02X}")),
        }
    }
    uri
}

/// The inverse, because `gsettings get` hands back the stored URL.
fn from_file_uri(value: &str) -> Option<PathBuf> {
    let encoded = value.strip_prefix("file://")?;
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    Some(PathBuf::from(String::from_utf8(decoded).ok()?))
}

/// `gsettings get` prints a quoted GVariant string.
fn unquote(value: &str) -> Option<&str> {
    value.trim().strip_prefix('\'')?.strip_suffix('\'')
}

/// The GNOME family spells the three placements differently; `Span` is the one case where
/// the extra name is worth keeping, since it means "one picture across all monitors".
fn gsettings_option(mode: Mode) -> &'static str {
    match mode {
        Mode::Fit => "scaled",
        Mode::Crop | Mode::SolarCrop => "zoom",
        Mode::Center => "centered",
        Mode::Span => "spanned",
    }
}

/// xfce4-desktop numbers its styles: 1 centred, 2 scaled, 4 zoomed.
fn xfce_style(mode: Mode) -> &'static str {
    match mode {
        Mode::Fit | Mode::Span => "2",
        Mode::Crop | Mode::SolarCrop => "4",
        Mode::Center => "1",
    }
}

fn sway_fit(mode: Mode) -> &'static str {
    match mode {
        Mode::Fit => "fit",
        Mode::Crop | Mode::SolarCrop => "fill",
        Mode::Center => "center",
        Mode::Span => "stretch",
    }
}

fn feh_flag(mode: Mode) -> &'static str {
    match mode {
        Mode::Fit => "--bg-max",
        Mode::Crop | Mode::Span | Mode::SolarCrop => "--bg-fill",
        Mode::Center => "--bg-center",
    }
}

fn xwallpaper_flag(mode: Mode) -> &'static str {
    match mode {
        Mode::Fit => "--maximize",
        Mode::Crop | Mode::Span | Mode::SolarCrop => "--zoom",
        Mode::Center => "--center",
    }
}

fn hsetroot_flag(mode: Mode) -> &'static str {
    match mode {
        Mode::Fit | Mode::Span => "-full",
        Mode::Crop | Mode::SolarCrop => "-fill",
        Mode::Center => "-center",
    }
}

/// XFCE keeps one property per monitor and workspace, so the picture property has to be
/// discovered before it can be set. `xfconf-query -l` prints one property per line.
fn xfce_targets(listed: &str) -> Vec<String> {
    listed
        .lines()
        .map(str::trim)
        .filter(|property| property.ends_with("/last-image"))
        .map(str::to_string)
        .collect()
}

fn xfce_chains(path: &Path, mode: Mode) -> Result<Vec<Vec<Step>>> {
    let path = path.to_string_lossy().into_owned();
    let listed = Command::new("xfconf-query")
        .args(["-c", XFCE, "-l"])
        .output()
        .map_err(|error| Error::Invalid(format!("xfconf-query is unavailable: {error}")))?;
    if !listed.status.success() {
        return Err(Error::Invalid(
            "xfconf-query could not list the desktop properties".into(),
        ));
    }
    let targets = xfce_targets(&String::from_utf8_lossy(&listed.stdout));
    if targets.is_empty() {
        return Err(Error::Invalid(
            "Xfce has no wallpaper property to set".into(),
        ));
    }
    let style = xfce_style(mode);
    let mut steps = Vec::with_capacity(targets.len() * 2);
    for target in &targets {
        steps.push(step(
            "xfconf-query",
            &["-c", XFCE, "-p", target, "-s", &path],
        ));
        let style_property = target.replace("/last-image", "/image-style");
        steps.push(soft(
            "xfconf-query",
            &["-c", XFCE, "-p", &style_property, "-s", style],
        ));
    }
    Ok(vec![steps])
}

/// Every chain is an alternative way to set the picture; the first one that runs to
/// completion wins.
fn plans(desktop: Desktop, path: &Path, mode: Mode) -> Vec<Vec<Step>> {
    let path_text = path.to_string_lossy().into_owned();
    let uri = file_uri(path);
    match desktop {
        Desktop::Kde => vec![vec![step("plasma-apply-wallpaperimage", &[&path_text])]],
        Desktop::Gnome => vec![vec![
            step("gsettings", &["set", GNOME, "picture-uri", &uri]),
            // Only newer GNOME has a separate dark-mode picture.
            soft("gsettings", &["set", GNOME, "picture-uri-dark", &uri]),
            step(
                "gsettings",
                &["set", GNOME, "picture-options", gsettings_option(mode)],
            ),
        ]],
        Desktop::Cinnamon => vec![vec![
            step("gsettings", &["set", CINNAMON, "picture-uri", &uri]),
            step(
                "gsettings",
                &["set", CINNAMON, "picture-options", gsettings_option(mode)],
            ),
        ]],
        Desktop::Mate => vec![vec![
            step("gsettings", &["set", MATE, "picture-filename", &path_text]),
            step(
                "gsettings",
                &["set", MATE, "picture-options", gsettings_option(mode)],
            ),
        ]],
        // Handled by `xfce_chains`, which needs the property list first.
        Desktop::Xfce => Vec::new(),
        Desktop::Sway => vec![vec![step(
            "swaymsg",
            &["output", "*", "bg", &path_text, sway_fit(mode)],
        )]],
        Desktop::Hyprland => vec![vec![step(
            "hyprctl",
            &["hyprpaper", "wallpaper", &format!(",{path_text}")],
        )]],
        Desktop::X11 => vec![
            vec![step("feh", &[feh_flag(mode), "--no-fehbg", &path_text])],
            vec![step("xwallpaper", &[xwallpaper_flag(mode), &path_text])],
            vec![step("hsetroot", &[hsetroot_flag(mode), &path_text])],
        ],
    }
}

/// Runs a chain to the end. A required step that fails (or a program that is not installed)
/// fails the whole chain, so the next alternative is tried with a real picture change.
fn run_chain(chain: &[Step]) -> std::result::Result<(), String> {
    for step in chain {
        let output = match Command::new(step.program).args(&step.args).output() {
            Ok(output) => output,
            Err(error) => return Err(format!("{}: {error}", step.program)),
        };
        if output.status.success() {
            continue;
        }
        if step.optional {
            continue;
        }
        let detail = String::from_utf8_lossy(&output.stderr);
        let detail = detail.trim();
        return Err(if detail.is_empty() {
            format!("{} refused the picture", step.program)
        } else {
            format!("{}: {detail}", step.program)
        });
    }
    Ok(())
}

pub fn apply(path: &Path, mode: Mode) -> Result<()> {
    if !path.is_file() {
        return Err(Error::Invalid("Wallpaper image is unavailable".into()));
    }
    let path = std::path::absolute(path)?;
    let desktop = detect(&|key| std::env::var(key).ok())
        .ok_or_else(|| Error::Invalid("No known desktop environment was detected".into()))?;
    let chains = if desktop == Desktop::Xfce {
        xfce_chains(&path, mode)?
    } else {
        plans(desktop, &path, mode)
    };
    let mut failures = Vec::new();
    for chain in &chains {
        match run_chain(chain) {
            Ok(()) => return Ok(()),
            Err(error) => failures.push(error),
        }
    }
    Err(Error::Invalid(format!(
        "No wallpaper tool accepted the picture ({})",
        failures.join("; ")
    )))
}

pub fn active() -> Option<PathBuf> {
    let desktop = detect(&|key| std::env::var(key).ok())?;
    match desktop {
        Desktop::Gnome => read_gsettings(GNOME, "picture-uri"),
        Desktop::Cinnamon => read_gsettings(CINNAMON, "picture-uri"),
        // MATE stores a plain filename rather than a URL.
        Desktop::Mate => read_gsettings(MATE, "picture-filename"),
        Desktop::Xfce => read_xfce(),
        _ => None,
    }
}

fn read_gsettings(schema: &str, key: &str) -> Option<PathBuf> {
    let output = Command::new("gsettings")
        .args(["get", schema, key])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    let value = unquote(&raw)?;
    if value.is_empty() {
        return None;
    }
    from_file_uri(value).or_else(|| Some(PathBuf::from(value)))
}

fn read_xfce() -> Option<PathBuf> {
    let listed = Command::new("xfconf-query")
        .args(["-c", XFCE, "-l"])
        .output()
        .ok()?;
    if !listed.status.success() {
        return None;
    }
    let first = xfce_targets(&String::from_utf8_lossy(&listed.stdout))
        .into_iter()
        .next()?;
    let value = Command::new("xfconf-query")
        .args(["-c", XFCE, "-p", &first])
        .output()
        .ok()?;
    if !value.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&value.stdout);
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    Some(PathBuf::from(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let owned: Vec<(String, String)> = pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();
        move |key| {
            owned
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value.clone())
        }
    }

    #[test]
    fn detection_prefers_the_specific_desktop_and_ignores_the_rest() {
        let gnome = session(&[("XDG_CURRENT_DESKTOP", "ubuntu:GNOME")]);
        assert_eq!(detect(&gnome), Some(Desktop::Gnome));

        let plasma = session(&[
            ("XDG_CURRENT_DESKTOP", "KDE"),
            ("DESKTOP_SESSION", "plasma"),
        ]);
        assert_eq!(detect(&plasma), Some(Desktop::Kde));

        let cinnamon = session(&[("DESKTOP_SESSION", "cinnamon")]);
        assert_eq!(detect(&cinnamon), Some(Desktop::Cinnamon));

        let mate = session(&[("XDG_CURRENT_DESKTOP", "MATE")]);
        assert_eq!(detect(&mate), Some(Desktop::Mate));

        let xfce = session(&[("DESKTOP_SESSION", "xfce")]);
        assert_eq!(detect(&xfce), Some(Desktop::Xfce));
    }

    #[test]
    fn wayland_compositors_are_recognised_by_name_or_by_their_socket() {
        let hyprland = session(&[("XDG_CURRENT_DESKTOP", "Hyprland")]);
        assert_eq!(detect(&hyprland), Some(Desktop::Hyprland));

        let sway = session(&[("SWAYSOCK", "/run/user/1000/sway-ipc.sock")]);
        assert_eq!(detect(&sway), Some(Desktop::Sway));

        let bare_x11 = session(&[("DISPLAY", ":0")]);
        assert_eq!(detect(&bare_x11), Some(Desktop::X11));

        assert_eq!(detect(&session(&[])), None);
    }

    #[test]
    fn a_space_in_the_path_survives_the_round_trip_through_a_file_url() {
        let path = Path::new("/home/ana/Pictures/Solar Observatory/x 1.jpg");
        let uri = file_uri(path);
        assert_eq!(
            uri,
            "file:///home/ana/Pictures/Solar%20Observatory/x%201.jpg"
        );
        assert_eq!(from_file_uri(&uri), Some(path.to_path_buf()));
        assert_eq!(from_file_uri("not a url"), None);
    }

    #[test]
    fn quoted_settings_values_are_unwrapped_once() {
        assert_eq!(unquote("'file:///a.jpg'\n"), Some("file:///a.jpg"));
        assert_eq!(unquote("''"), Some(""));
        assert_eq!(unquote("file:///a.jpg"), None);
    }

    #[test]
    fn gnome_family_gets_the_uri_and_the_matching_option() {
        let chains = plans(Desktop::Gnome, Path::new("/a/b.jpg"), Mode::Crop);
        let steps = &chains[0];
        assert_eq!(
            steps[0].args,
            ["set", GNOME, "picture-uri", "file:///a/b.jpg"]
        );
        assert!(steps[1].optional, "the dark-mode picture is optional");
        assert_eq!(steps[2].args, ["set", GNOME, "picture-options", "zoom"]);

        let spanned = plans(Desktop::Gnome, Path::new("/a/b.jpg"), Mode::Span);
        assert_eq!(spanned[0][2].args[3], "spanned");
    }

    #[test]
    fn each_desktop_gets_its_own_vocabulary() {
        let path = Path::new("/a/b.jpg");
        let sway = plans(Desktop::Sway, path, Mode::Fit);
        assert_eq!(sway[0][0].args, ["output", "*", "bg", "/a/b.jpg", "fit"]);

        let cairo = plans(Desktop::X11, path, Mode::Crop);
        assert_eq!(cairo[0][0].args, ["--bg-fill", "--no-fehbg", "/a/b.jpg"]);
        assert_eq!(cairo[1][0].program, "xwallpaper");
        assert_eq!(cairo[2][0].program, "hsetroot");

        let plasma = plans(Desktop::Kde, path, Mode::Center);
        assert_eq!(plasma[0][0].program, "plasma-apply-wallpaperimage");
        assert_eq!(plasma[0][0].args, ["/a/b.jpg"]);
    }

    #[test]
    fn xfce_properties_are_discovered_before_they_are_set() {
        let listed = "/backdrop/screen0/monitor0/workspace0/last-image\n\
                      /backdrop/screen0/monitor0/workspace0/image-style\n\
                      /backdrop/screen0/monitor1/workspace0/last-image\n\
                      /desktop-icons/style\n";
        let targets = xfce_targets(listed);
        assert_eq!(
            targets,
            vec![
                String::from("/backdrop/screen0/monitor0/workspace0/last-image"),
                String::from("/backdrop/screen0/monitor1/workspace0/last-image"),
            ]
        );
        assert_eq!(xfce_style(Mode::Center), "1");
        assert_eq!(xfce_style(Mode::Crop), "4");
        assert_eq!(xfce_style(Mode::Fit), "2");
    }

    #[test]
    fn placements_keep_their_meaning_across_tools() {
        assert_eq!(sway_fit(Mode::Span), "stretch");
        assert_eq!(feh_flag(Mode::Fit), "--bg-max");
        assert_eq!(xwallpaper_flag(Mode::Center), "--center");
        assert_eq!(hsetroot_flag(Mode::Crop), "-fill");
    }
}

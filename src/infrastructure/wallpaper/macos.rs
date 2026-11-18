//! macOS is the one system where the picture is set per screen, on the main thread, through
//! `NSWorkspace`. Everything here therefore crosses a thread boundary: the app applies from a
//! worker, AppKit answers only on the main queue.
//!
//! The native call reports success even when recent builds quietly keep the old picture, so
//! the desktop is read back afterwards and System Events is the fallback.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use dispatch2::DispatchQueue;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{
    NSImageScaling, NSScreen, NSWorkspace, NSWorkspaceDesktopImageAllowClippingKey,
    NSWorkspaceDesktopImageOptionKey, NSWorkspaceDesktopImageScalingKey,
};
use objc2_foundation::{NSDictionary, NSNumber, NSString, NSURL};

use super::{Placement, applescript_literal, placement};
use crate::{Error, Result, domain::Mode};

/// A request that waits forever is worse than a request that fails: if nothing is draining
/// the main queue (a headless session, or a test harness, for instance) the app must keep
/// going. A real round trip takes microseconds.
const MAIN_QUEUE_TIMEOUT: Duration = Duration::from_secs(2);

/// Runs `work` on the main thread and brings its value back.
fn on_main<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Result<T> {
    // Dispatching to the queue we are standing on would deadlock, and every AppKit call
    // below is just as happy to run here.
    if MainThreadMarker::new().is_some() {
        return Ok(work());
    }
    let (send, receive) = mpsc::channel();
    DispatchQueue::main().exec_async(move || {
        let _ = send.send(work());
    });
    receive
        .recv_timeout(MAIN_QUEUE_TIMEOUT)
        .map_err(|_| Error::Invalid("The main thread did not answer the wallpaper request".into()))
}

pub fn apply(path: &Path, mode: Mode) -> Result<()> {
    if !path.is_file() {
        return Err(Error::Invalid("Wallpaper image is unavailable".into()));
    }
    let path = std::path::absolute(path)?;
    let target = path.clone();
    let native = on_main(move || set(&target, mode))?;
    if native.is_ok() && wait_for(&path) {
        return Ok(());
    }
    let fallback = system_events(&path);
    match (native, fallback) {
        (_, Ok(())) => Ok(()),
        (Ok(()), Err(fallback)) => Err(Error::Invalid(format!(
            "The desktop kept the previous picture and System Events failed: {fallback}"
        ))),
        (Err(native), Err(fallback)) => Err(Error::Invalid(format!(
            "macOS rejected the wallpaper ({native}) and System Events failed: {fallback}"
        ))),
    }
}

pub fn active() -> Option<PathBuf> {
    let current = on_main(|| {
        let marker = MainThreadMarker::new()?;
        let screen = NSScreen::mainScreen(marker)?;
        NSWorkspace::sharedWorkspace()
            .desktopImageURLForScreen(&screen)?
            .to_file_path()
    });
    current.ok().flatten()
}

/// Main thread only.
fn set(path: &Path, mode: Mode) -> Result<()> {
    let marker = MainThreadMarker::new().ok_or_else(|| {
        Error::Invalid("macOS needs the main thread to change the desktop".into())
    })?;
    let url = NSURL::from_file_path(path)
        .ok_or_else(|| Error::Invalid("Wallpaper path is not a local file".into()))?;
    let options = options(mode);
    let workspace = NSWorkspace::sharedWorkspace();
    let screens = NSScreen::screens(marker);
    if screens.is_empty() {
        return Err(Error::Invalid(
            "There are no screens to receive the wallpaper".into(),
        ));
    }
    for screen in screens.iter() {
        // SAFETY: `options` carries the documented scaling and clipping keys with `NSNumber`
        // values, which is exactly what this selector expects.
        unsafe { workspace.setDesktopImageURL_forScreen_options_error(&url, &screen, &options) }
            .map_err(|error| Error::Invalid(format!("macOS rejected the wallpaper: {error}")))?;
    }
    Ok(())
}

/// The option dictionary AppKit expects, built from the keys it exports. `Fill` scales the
/// picture until it covers the screen and clips the overflow; `Fit` keeps all of it visible;
/// `Center` leaves it at its natural size.
fn options(mode: Mode) -> Retained<NSDictionary<NSWorkspaceDesktopImageOptionKey, AnyObject>> {
    let placement = placement(mode);
    let scaling = match placement {
        Placement::Center => NSImageScaling::ScaleNone,
        Placement::Fit | Placement::Fill => NSImageScaling::ScaleProportionallyUpOrDown,
    };
    let numbers = [
        NSNumber::new_usize(scaling.0),
        NSNumber::new_bool(matches!(placement, Placement::Fill)),
    ];
    // SAFETY: both statics are the option keys AppKit exports for this dictionary.
    let keys = unsafe {
        [
            NSWorkspaceDesktopImageScalingKey,
            NSWorkspaceDesktopImageAllowClippingKey,
        ]
    };
    let values = [&*numbers[0], &*numbers[1]];
    let numbers = NSDictionary::<NSString, NSNumber>::from_slices(&keys, &values);
    // SAFETY: the keys are already `NSString` and the values are `NSNumber`, so only the
    // type-erased value slot differs from the dictionary that AppKit declares.
    unsafe {
        Retained::cast_unchecked::<NSDictionary<NSWorkspaceDesktopImageOptionKey, AnyObject>>(
            numbers,
        )
    }
}

/// Main thread only: whether every screen reports `path` as its picture.
fn shows(path: &Path) -> bool {
    let Some(marker) = MainThreadMarker::new() else {
        return false;
    };
    let workspace = NSWorkspace::sharedWorkspace();
    let screens = NSScreen::screens(marker);
    !screens.is_empty()
        && screens.iter().all(|screen| {
            workspace
                .desktopImageURLForScreen(&screen)
                .and_then(|url| url.to_file_path())
                .is_some_and(|current| current == path)
        })
}

/// The read-back is not instant on every build, so it is retried before giving up.
fn wait_for(path: &Path) -> bool {
    for attempt in 1..=3u64 {
        let target = path.to_path_buf();
        if on_main(move || shows(&target)).unwrap_or(false) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(120 * attempt));
    }
    false
}

/// The older path. It needs Automation permission for System Events, which is why it only
/// runs after the native call did not take.
fn system_events(path: &Path) -> Result<()> {
    let path = path
        .to_str()
        .ok_or_else(|| Error::Invalid("Wallpaper path is not valid UTF-8".into()))?;
    let script = format!(
        "tell application \"System Events\" to tell every desktop to set picture to {}",
        applescript_literal(path)
    );
    let output = std::process::Command::new("osascript")
        .args(["-e", script.as_str()])
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr);
    Err(Error::Invalid(format!(
        "System Events refused the wallpaper: {}",
        detail.trim()
    )))
}

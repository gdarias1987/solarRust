//! Windows drives the desktop through the same COM object the Settings app uses, so the
//! position and the image travel together and the change lands on every monitor.

use crate::{Error, Result, domain::Mode};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::{
    Win32::{
        System::Com::{
            CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
            CoUninitialize,
        },
        UI::Shell::{
            DESKTOP_WALLPAPER_POSITION, DWPOS_CENTER, DWPOS_FILL, DWPOS_FIT, DWPOS_SPAN,
            DesktopWallpaper, IDesktopWallpaper,
        },
    },
    core::PCWSTR,
};

fn with_wallpaper<T>(f: impl FnOnce(&IDesktopWallpaper) -> Result<T>) -> Result<T> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|e| Error::Invalid(format!("Windows COM initialization failed: {e}")))?;
        let result = (|| {
            let wallpaper: IDesktopWallpaper =
                CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL).map_err(|e| {
                    Error::Invalid(format!("Windows wallpaper service unavailable: {e}"))
                })?;
            f(&wallpaper)
        })();
        CoUninitialize();
        result
    }
}

fn position(mode: Mode) -> DESKTOP_WALLPAPER_POSITION {
    match mode {
        Mode::Fit => DWPOS_FIT,
        Mode::Crop | Mode::SolarCrop => DWPOS_FILL,
        Mode::Center => DWPOS_CENTER,
        Mode::Span => DWPOS_SPAN,
    }
}

pub fn apply(path: &Path, mode: Mode) -> Result<()> {
    if !path.is_file() {
        return Err(Error::Invalid("Wallpaper image is unavailable".into()));
    }
    let path = std::path::absolute(path)?;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    with_wallpaper(|wallpaper| unsafe {
        wallpaper
            .SetPosition(position(mode))
            .map_err(|e| Error::Invalid(format!("Unable to set wallpaper position: {e}")))?;
        wallpaper
            .SetWallpaper(PCWSTR::null(), PCWSTR(wide.as_ptr()))
            .map_err(|e| Error::Invalid(format!("Unable to set wallpaper: {e}")))?;
        Ok(())
    })
}

pub fn active() -> Option<std::path::PathBuf> {
    with_wallpaper(|wallpaper| unsafe {
        let path = wallpaper
            .GetWallpaper(PCWSTR::null())
            .map_err(|e| Error::Invalid(e.to_string()))?;
        if path.is_null() {
            return Err(Error::Invalid("No active wallpaper path".into()));
        }
        let value = path.to_string().map_err(|e| Error::Invalid(e.to_string()));
        CoTaskMemFree(Some(path.0.cast()));
        value.map(std::path::PathBuf::from)
    })
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_wallpaper_modes_to_windows_positions() {
        assert_eq!(position(Mode::Fit), DWPOS_FIT);
        assert_eq!(position(Mode::Crop), DWPOS_FILL);
        assert_eq!(position(Mode::SolarCrop), DWPOS_FILL);
        assert_eq!(position(Mode::Center), DWPOS_CENTER);
        assert_eq!(position(Mode::Span), DWPOS_SPAN);
    }

    #[test]
    #[ignore = "changes the real desktop temporarily; run only with explicit approval"]
    fn windows_wallpaper_round_trip() {
        use windows::Win32::UI::Shell::DSS_SLIDESHOW;
        let image = std::path::absolute("content/previews/latest_256_0304.jpg").unwrap();
        with_wallpaper(|desktop| unsafe {
            let count = desktop
                .GetMonitorDevicePathCount()
                .map_err(|e| Error::Invalid(e.to_string()))?;
            if count == 0 {
                return Err(Error::Invalid("No monitors available".into()));
            }
            if desktop
                .GetStatus()
                .map_err(|e| Error::Invalid(e.to_string()))?
                .0
                & DSS_SLIDESHOW.0
                != 0
            {
                return Err(Error::Invalid(
                    "Round-trip test does not alter slideshow configuration".into(),
                ));
            }
            let old_position = desktop
                .GetPosition()
                .map_err(|e| Error::Invalid(e.to_string()))?;
            let mut originals = Vec::new();
            for i in 0..count {
                let monitor = desktop
                    .GetMonitorDevicePathAt(i)
                    .map_err(|e| Error::Invalid(e.to_string()))?;
                if monitor.is_null() {
                    return Err(Error::Invalid("Monitor path unavailable".into()));
                }
                let monitor_string = monitor.to_string();
                CoTaskMemFree(Some(monitor.0.cast()));
                let monitor_string = monitor_string.map_err(|e| Error::Invalid(e.to_string()))?;
                let monitor_wide: Vec<u16> = monitor_string.encode_utf16().chain(Some(0)).collect();
                let original = desktop
                    .GetWallpaper(PCWSTR(monitor_wide.as_ptr()))
                    .map_err(|e| Error::Invalid(e.to_string()))?;
                if original.is_null() {
                    return Err(Error::Invalid("Original wallpaper is unavailable".into()));
                }
                let original_string = original.to_string();
                CoTaskMemFree(Some(original.0.cast()));
                let original_string = original_string.map_err(|e| Error::Invalid(e.to_string()))?;
                eprintln!("Monitor {i}: {monitor_string}; wallpaper: {original_string}");
                if !Path::new(&original_string).is_file() {
                    return Err(Error::Invalid(
                        "Original wallpaper file is unavailable".into(),
                    ));
                }
                originals.push((
                    monitor_wide,
                    original_string
                        .encode_utf16()
                        .chain(Some(0))
                        .collect::<Vec<u16>>(),
                ));
            }
            let result = (|| -> Result<()> {
                apply(&image, Mode::Fit)?;
                std::thread::sleep(std::time::Duration::from_secs(2));
                for (monitor, _) in &originals {
                    let p = desktop
                        .GetWallpaper(PCWSTR(monitor.as_ptr()))
                        .map_err(|e| Error::Invalid(e.to_string()))?;
                    if !p.is_null() {
                        eprintln!("After set: {:?}", p.to_string());
                        CoTaskMemFree(Some(p.0.cast()));
                    }
                }
                let current = desktop
                    .GetWallpaper(PCWSTR(originals[0].0.as_ptr()))
                    .map_err(|e| Error::Invalid(e.to_string()))?;
                if current.is_null() {
                    return Err(Error::Invalid("Windows returned no wallpaper".into()));
                }
                let value = current.to_string();
                CoTaskMemFree(Some(current.0.cast()));
                let value = value.map_err(|e| Error::Invalid(e.to_string()))?;
                if !value.eq_ignore_ascii_case(&image.to_string_lossy()) {
                    return Err(Error::Invalid(format!(
                        "Unexpected wallpaper path: {value}"
                    )));
                }
                Ok(())
            })();
            let mut restore_error = desktop.SetPosition(old_position).err();
            for (monitor, original) in &originals {
                if let Err(e) =
                    desktop.SetWallpaper(PCWSTR(monitor.as_ptr()), PCWSTR(original.as_ptr()))
                {
                    restore_error = Some(e);
                }
            }
            if let Some(e) = restore_error {
                return Err(Error::Invalid(format!(
                    "Failed to restore original wallpaper: {e}"
                )));
            }
            result
        })
        .unwrap();
    }
}

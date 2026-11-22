//! Default background: the user's current Windows wallpaper.

use std::path::PathBuf;

use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};
use windows::core::PCWSTR;

/// Path of the wallpaper configured in Windows, if it is a real file.
pub fn desktop_wallpaper() -> Option<PathBuf> {
    let mut buffer = [0u16; 1024];
    // SAFETY: `buffer` is a valid, writable MAX_PATH-sized UTF-16 buffer and
    // the API only writes a path into it.
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            buffer.len() as u32,
            Some(buffer.as_mut_ptr().cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    if ok.is_err() {
        return None;
    }

    let len = buffer.iter().position(|&c| c == 0).unwrap_or(0);
    if len == 0 {
        return None;
    }
    let path = PathBuf::from(String::from_utf16_lossy(&buffer[..len]));
    path.is_file().then_some(path)
}

/// Keeps the unused-import linter honest on builds where the feature is off.
#[allow(dead_code)]
fn _keep_pcwstr(_: PCWSTR) {}

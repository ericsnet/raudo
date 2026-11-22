//! Drive enumeration for the sidebar and the home page.

use windows::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetLogicalDrives, GetVolumeInformationW,
};
use windows::core::PCWSTR;

#[derive(Debug, Clone)]
pub struct DriveInfo {
    /// `C:\`, ready to be used as a directory.
    pub root: String,
    /// `C:` for labels and cards.
    pub letter: String,
    pub label: String,
    pub total: u64,
    pub free: u64,
}

/// Every drive that is ready to be read, in letter order.
pub fn drives() -> Vec<DriveInfo> {
    // SAFETY: no pointer arguments.
    let mask = unsafe { GetLogicalDrives() };
    let mut result = Vec::new();

    for index in 0..26u32 {
        if mask & (1 << index) == 0 {
            continue;
        }
        let letter = (b'A' + index as u8) as char;
        let root = format!("{letter}:\\");
        let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();

        let mut free = 0u64;
        let mut total = 0u64;
        // SAFETY: `wide` is NUL-terminated; both out-pointers are valid.
        let ok = unsafe {
            GetDiskFreeSpaceExW(
                PCWSTR(wide.as_ptr()),
                Some(&mut free),
                Some(&mut total),
                None,
            )
        };
        if ok.is_err() || total == 0 {
            // Not ready (empty card reader, disconnected network drive...).
            continue;
        }

        result.push(DriveInfo {
            root,
            letter: format!("{letter}:"),
            label: volume_label(&wide).unwrap_or_else(|| "Disco local".to_string()),
            total,
            free,
        });
    }

    result
}

fn volume_label(root: &[u16]) -> Option<String> {
    let mut buffer = [0u16; 64];
    // SAFETY: the label buffer is valid for `buffer.len()` UTF-16 units.
    let ok = unsafe {
        GetVolumeInformationW(
            PCWSTR(root.as_ptr()),
            Some(&mut buffer[..]),
            None,
            None,
            None,
            None,
        )
    };
    if ok.is_err() {
        return None;
    }
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(0);
    if len == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buffer[..len]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_at_least_one_drive() {
        let list = drives();
        assert!(
            !list.is_empty(),
            "a Windows machine has at least one ready drive"
        );
        assert!(list.iter().all(|d| d.root.ends_with(":\\")));
        assert!(list.iter().all(|d| d.free <= d.total));
    }
}

//! Directory enumeration built directly on `FindFirstFileExW`.
//!
//! The shell is deliberately not involved: no `IShellFolder`, no
//! `SHGetFileInfo`, no per-item property lookups. A directory listing is a
//! single kernel walk with large fetch buffers.

use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use windows::Win32::Foundation::{FILETIME, GetLastError, HANDLE, SYSTEMTIME};
use windows::Win32::Storage::FileSystem::{
    FIND_FIRST_EX_FLAGS, FileTimeToLocalFileTime, FindClose, FindExInfoBasic,
    FindExSearchNameMatch, FindFirstFileExW, FindNextFileW, WIN32_FIND_DATAW,
};
use windows::Win32::System::Time::FileTimeToSystemTime;
use windows::core::PCWSTR;

/// Return basic data (name, size, timestamps, attributes) instead of the
/// full `WIN32_FIND_DATA` set. Cuts the cost of each directory entry.
const FIND_FIRST_EX_LARGE_FETCH: u32 = 0x0000_0002;

// Attribute bits, kept local so a `windows` crate version bump cannot change
// the meaning of the comparisons below.
const ATTR_DIRECTORY: u32 = 0x0000_0010;
const ATTR_HIDDEN: u32 = 0x0000_0002;
const ATTR_SYSTEM: u32 = 0x0000_0004;
const ATTR_REPARSE_POINT: u32 = 0x0000_0400;

/// Seconds between the Windows epoch (1601-01-01) and the Unix epoch.
const WINDOWS_EPOCH_OFFSET_SECS: u64 = 11_644_473_600;

/// One directory entry, already ordered and ready for the UI.
#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub is_hidden: bool,
    pub is_reparse_point: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    /// Local-time date shown in the file list, e.g. `22/11/2026 13:05`.
    pub modified_text: String,
}

/// Column used to order a directory listing.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name = 0,
    Size = 1,
    Modified = 2,
}

/// Ordering direction inside each group (directories are always first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

/// A directory could not be listed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListError {
    pub code: u32,
}

impl std::fmt::Display for ListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no se pudo leer la carpeta (error {})", self.code)
    }
}

impl std::error::Error for ListError {}

/// Lists `path`, directories first and then by case-insensitive name.
pub fn list_dir(path: &Path) -> Result<Vec<Entry>, ListError> {
    let mut pattern: Vec<u16> = path.as_os_str().encode_wide().collect();
    if !pattern.ends_with(&[b'\\' as u16]) {
        pattern.push(b'\\' as u16);
    }
    pattern.push(b'*' as u16);
    pattern.push(0);

    let mut data = WIN32_FIND_DATAW::default();
    // SAFETY: `pattern` is NUL-terminated and outlives the call; `data` is a
    // valid, writable `WIN32_FIND_DATAW`.
    let handle = unsafe {
        FindFirstFileExW(
            PCWSTR(pattern.as_ptr()),
            FindExInfoBasic,
            (&mut data as *mut WIN32_FIND_DATAW).cast(),
            FindExSearchNameMatch,
            None,
            FIND_FIRST_EX_FLAGS(FIND_FIRST_EX_LARGE_FETCH),
        )
    };

    let handle: HANDLE = match handle {
        Ok(handle) => handle,
        Err(_) => {
            let code = unsafe { GetLastError() }.0;
            return Err(ListError { code });
        }
    };

    let mut entries = Vec::with_capacity(256);
    loop {
        if let Some(entry) = entry_from(&data) {
            entries.push(entry);
        }
        // SAFETY: `handle` came from a successful `FindFirstFileExW`; `data`
        // is reused as the output buffer, which the API expects.
        if unsafe { FindNextFileW(handle, &mut data) }.is_err() {
            break;
        }
    }

    // SAFETY: `handle` is a live search handle and is not used afterwards.
    unsafe {
        let _ = FindClose(handle);
    }

    entries.sort_by(entry_name_order);

    Ok(entries)
}

fn entry_name_order(a: &Entry, b: &Entry) -> std::cmp::Ordering {
    b.is_dir
        .cmp(&a.is_dir)
        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
}

/// Re-orders an already loaded listing. Directories stay grouped first.
pub fn sort_entries(entries: &mut [Entry], key: SortKey, dir: SortDir) {
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then_with(|| {
            let order = match key {
                SortKey::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortKey::Size => a.size.cmp(&b.size),
                SortKey::Modified => a.modified.cmp(&b.modified),
            };
            match dir {
                SortDir::Asc => order,
                SortDir::Desc => order.reverse(),
            }
        })
    });
}

fn entry_from(data: &WIN32_FIND_DATAW) -> Option<Entry> {
    let len = data
        .cFileName
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(data.cFileName.len());
    if len == 0 {
        return None;
    }

    let name = OsString::from_wide(&data.cFileName[..len])
        .to_string_lossy()
        .into_owned();
    if name == "." || name == ".." {
        return None;
    }

    let attrs = data.dwFileAttributes;
    let is_dir = attrs & ATTR_DIRECTORY != 0;

    Some(Entry {
        name,
        is_dir,
        is_hidden: attrs & (ATTR_HIDDEN | ATTR_SYSTEM) != 0,
        is_reparse_point: attrs & ATTR_REPARSE_POINT != 0,
        size: if is_dir {
            0
        } else {
            ((data.nFileSizeHigh as u64) << 32) | data.nFileSizeLow as u64
        },
        modified: filetime_to_system_time(&data.ftLastWriteTime),
        modified_text: filetime_text(&data.ftLastWriteTime),
    })
}

/// Local date and time for the file list, or an empty string when the
/// timestamp cannot be converted.
fn filetime_text(ft: &FILETIME) -> String {
    let mut local = FILETIME::default();
    let mut st = SYSTEMTIME::default();
    // SAFETY: both pointers reference writable, correctly sized structs.
    let converted = unsafe {
        FileTimeToLocalFileTime(ft, &mut local).is_ok()
            && FileTimeToSystemTime(&local, &mut st).is_ok()
    };
    if !converted {
        return String::new();
    }
    format!(
        "{:02}/{:02}/{:04} {:02}:{:02}",
        st.wDay, st.wMonth, st.wYear, st.wHour, st.wMinute
    )
}

fn filetime_to_system_time(ft: &FILETIME) -> Option<SystemTime> {
    let ticks = ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64;
    if ticks == 0 {
        return None;
    }
    let secs = ticks / 10_000_000;
    if secs < WINDOWS_EPOCH_OFFSET_SECS {
        return None;
    }
    Some(UNIX_EPOCH + Duration::from_secs(secs - WINDOWS_EPOCH_OFFSET_SECS))
}

/// Human-readable size, e.g. `4,3 MB`.
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit]).replace('.', ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_sizes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1,0 KB");
        assert_eq!(format_size(1536), "1,5 KB");
        assert_eq!(format_size(5 * 1024 * 1024), "5,0 MB");
    }

    #[test]
    fn lists_windows_directory() {
        let entries = list_dir(Path::new("C:\\Windows")).expect("C:\\Windows should be listable");
        assert!(!entries.is_empty());
        assert!(entries.iter().all(|e| e.name != "." && e.name != ".."));
    }

    #[test]
    fn reports_missing_directory() {
        let err = list_dir(Path::new("C:\\raudo-does-not-exist")).unwrap_err();
        assert_ne!(err.code, 0);
    }

    #[test]
    fn directories_come_first() {
        let entries = list_dir(Path::new("C:\\Windows")).unwrap();
        let first_file = entries.iter().position(|e| !e.is_dir);
        let last_dir = entries.iter().rposition(|e| e.is_dir);
        if let (Some(file), Some(dir)) = (first_file, last_dir) {
            assert!(dir < file, "directories must be grouped before files");
        }
    }
}

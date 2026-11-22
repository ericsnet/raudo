//! Shell actions: open, open with, properties, recycle bin and terminal.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::UI::Shell::{
    FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_SILENT, SHFILEOPSTRUCTW, SHFileOperationW,
    ShellExecuteW,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn execute(verb: &str, path: &Path) {
    let verb = wide(verb);
    let file = wide_path(path);
    // SAFETY: both strings are NUL-terminated and live for the call.
    unsafe {
        let _ = ShellExecuteW(
            None,
            windows::core::PCWSTR(verb.as_ptr()),
            windows::core::PCWSTR(file.as_ptr()),
            windows::core::PCWSTR::null(),
            windows::core::PCWSTR::null(),
            SW_SHOWNORMAL,
        );
    }
}

/// Opens a file with its default application.
pub fn open(path: &Path) {
    execute("open", path);
}

/// Shows the "Open with" dialog.
pub fn open_with(path: &Path) {
    execute("openas", path);
}

/// Shows the Windows properties sheet.
pub fn show_properties(path: &Path) {
    execute("properties", path);
}

/// Sends a file or folder to the recycle bin, with the usual confirmation.
pub fn delete_to_recycle_bin(path: &Path) -> Result<(), u32> {
    let mut from: Vec<u16> = path.as_os_str().encode_wide().collect();
    from.push(0);
    from.push(0);

    let mut operation = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: windows::core::PCWSTR(from.as_ptr()),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT).0 as u16,
        ..Default::default()
    };

    // SAFETY: `operation` points at a double NUL-terminated buffer that lives
    // for the whole call.
    let result = unsafe { SHFileOperationW(&mut operation) };
    if result == 0 {
        Ok(())
    } else {
        Err(result as u32)
    }
}

/// Opens a terminal in `dir`.
pub fn open_terminal(dir: &Path) {
    let ok = std::process::Command::new("wt.exe")
        .arg("-d")
        .arg(dir)
        .spawn()
        .is_ok();
    if !ok {
        let _ = std::process::Command::new("powershell.exe")
            .arg("-NoExit")
            .arg("-Command")
            .arg(format!("Set-Location -LiteralPath '{}'", dir.display()))
            .spawn();
    }
}

/// Opens `cmd.exe` in `dir`.
pub fn open_cmd(dir: &Path) {
    let _ = std::process::Command::new("cmd.exe")
        .arg("/K")
        .arg("cd")
        .arg("/d")
        .arg(dir)
        .spawn();
}

/// Opens PowerShell in `dir`.
pub fn open_powershell(dir: &Path) {
    let _ = std::process::Command::new("powershell.exe")
        .arg("-NoExit")
        .arg("-Command")
        .arg(format!("Set-Location -LiteralPath '{}'", dir.display()))
        .spawn();
}

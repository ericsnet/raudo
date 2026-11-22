#![windows_subsystem = "windows"]

use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn main() -> Result<(), raudo_ui::PlatformError> {
    install_panic_log();

    let started = Instant::now();

    // Slint's software renderer shows the first frame ~10x faster and uses
    // noticeably less memory than the OpenGL one. The UI is plain enough that
    // it does not need GPU compositing yet; `SLINT_BACKEND` still overrides.
    if std::env::var_os("SLINT_BACKEND").is_none() {
        // SAFETY: called before any other thread exists.
        unsafe { std::env::set_var("SLINT_BACKEND", "winit-software") };
    }

    // The bench launches the binary without a console, so tracing is written
    // to a file. Interactive runs leave it unset.
    let trace = std::env::var_os("RAUDO_TRACE").is_some().then(|| {
        std::env::var_os("RAUDO_TRACE_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("raudo-trace.txt"))
    });

    // No argument opens the home page; a path opens that folder.
    let start_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .filter(|path| path.is_dir());

    place_window();

    raudo_ui::run(start_dir, trace, started)
}

fn log_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|dir| PathBuf::from(dir).join("Raudo"))
}

fn install_panic_log() {
    std::panic::set_hook(Box::new(|info| {
        let Some(dir) = log_dir() else { return };
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("crash.log"))
        {
            let _ = writeln!(file, "panic: {info}");
            let _ = writeln!(
                file,
                "backtrace:\n{}",
                std::backtrace::Backtrace::force_capture()
            );
        }
    }));
}

/// Window size and centering are applied natively: Slint auto-sizes the window
/// to its layout, which fights any size requested from Slint itself.
fn place_window() {
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(600));
        #[cfg(windows)]
        unsafe {
            use windows::Win32::Foundation::{HWND, LPARAM};
            use windows::Win32::UI::WindowsAndMessaging::{
                EnumWindows, GetSystemMetrics, IsWindowVisible, SM_CXSCREEN, SM_CYSCREEN,
                SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos,
            };
            use windows::core::BOOL;

            struct Search {
                pid: u32,
                found: HWND,
            }

            unsafe extern "system" fn callback(window: HWND, lparam: LPARAM) -> BOOL {
                let search = unsafe { &mut *(lparam.0 as *mut Search) };
                let mut process_id = 0u32;
                unsafe {
                    windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(
                        window,
                        Some(&mut process_id),
                    );
                }
                if process_id == search.pid && unsafe { IsWindowVisible(window) }.as_bool() {
                    search.found = window;
                    return BOOL(0);
                }
                BOOL(1)
            }

            let mut search = Search {
                pid: std::process::id(),
                found: HWND::default(),
            };
            let _ = EnumWindows(Some(callback), LPARAM(&mut search as *mut _ as isize));
            if search.found.is_invalid() {
                return;
            }

            let width = 1280;
            let height = 800;
            let x = (GetSystemMetrics(SM_CXSCREEN) - width) / 2;
            let y = (GetSystemMetrics(SM_CYSCREEN) - height) / 2;
            let _ = SetWindowPos(
                search.found,
                None,
                x,
                y,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    });
}

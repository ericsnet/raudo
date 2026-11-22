#![windows_subsystem = "windows"]

use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

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

    raudo_ui::run(start_dir, trace, started)
}

fn install_panic_log() {
    std::panic::set_hook(Box::new(|info| {
        let Some(appdata) = std::env::var_os("APPDATA") else {
            return;
        };
        let dir = PathBuf::from(appdata).join("Raudo");
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

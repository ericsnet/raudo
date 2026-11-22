#![windows_subsystem = "windows"]

use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<(), raudo_ui::PlatformError> {
    let started = Instant::now();

    // The bench launches the binary without a console, so tracing is written
    // to a file. Interactive runs leave it unset.
    let trace = std::env::var_os("RAUDO_TRACE").is_some().then(|| {
        std::env::var_os("RAUDO_TRACE_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("raudo-trace.txt"))
    });

    let start_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("C:\\"));

    raudo_ui::run(start_dir, trace, started)
}

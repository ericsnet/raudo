//! Measurement harness. Every number printed here is measured, never guessed.
//!
//! Usage:
//!   raudo-bench list <dir> [runs]
//!   raudo-bench startup <raudo.exe> [dir]
//!   raudo-bench all <raudo.exe> [dir]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first().map(String::as_str) else {
        usage();
        return;
    };

    match command {
        "list" => {
            let dir = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("C:\\Windows\\System32"));
            let runs = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(6usize);
            bench_list(&dir, runs);
        }
        "startup" => {
            let Some(exe) = args.get(1).map(PathBuf::from) else {
                usage();
                return;
            };
            let dir = args
                .get(2)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("C:\\Windows\\System32"));
            bench_startup(&exe, &dir);
        }
        "all" => {
            let Some(exe) = args.get(1).map(PathBuf::from) else {
                usage();
                return;
            };
            let dir = args
                .get(2)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("C:\\Windows\\System32"));
            bench_startup(&exe, &dir);
            println!();
            bench_list(&dir, 6);
        }
        _ => usage(),
    }
}

fn usage() {
    println!("raudo-bench list <dir> [runs]");
    println!("raudo-bench startup <raudo.exe> [dir]");
    println!("raudo-bench all <raudo.exe> [dir]");
}

fn bench_list(dir: &Path, runs: usize) {
    // One throwaway pass so measurement is not dominated by first-touch page
    // faults, then `runs` timed passes.
    let _ = raudo_core::list_dir(dir);

    let mut samples = Vec::with_capacity(runs);
    let mut entries = 0usize;
    let mut dirs = 0usize;
    for _ in 0..runs {
        let start = Instant::now();
        match raudo_core::list_dir(dir) {
            Ok(list) => {
                samples.push(start.elapsed().as_secs_f64() * 1000.0);
                entries = list.len();
                dirs = list.iter().filter(|e| e.is_dir).count();
            }
            Err(err) => {
                println!("error listing {}: {err}", dir.display());
                return;
            }
        }
    }

    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = samples[samples.len() / 2];
    let best = samples[0];
    let worst = samples[samples.len() - 1];
    let total: f64 = samples.iter().sum();

    println!("dir            {}", dir.display());
    println!(
        "entries        {entries} ({dirs} folders, {} files)",
        entries - dirs
    );
    println!("listing        best {best:.2} ms | median {median:.2} ms | worst {worst:.2} ms");
    println!(
        "listing avg    {:.2} ms over {runs} runs",
        total / runs as f64
    );
}

fn bench_startup(exe: &Path, dir: &Path) {
    let trace = std::env::temp_dir().join(format!("raudo-trace-{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&trace);

    let started = Instant::now();
    let mut child = match Command::new(exe)
        .arg(dir)
        .env("RAUDO_TRACE", "1")
        .env("RAUDO_TRACE_FILE", &trace)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            println!("could not launch {}: {err}", exe.display());
            return;
        }
    };

    let deadline = Instant::now() + Duration::from_secs(15);
    let mut first_frame = None;
    while Instant::now() < deadline {
        if let Ok(text) = std::fs::read_to_string(&trace)
            && let Some(value) = text.trim().strip_prefix("first_frame_ms=")
            && let Ok(ms) = value.parse::<u128>()
        {
            first_frame = Some(ms);
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let wall = started.elapsed().as_millis();

    std::thread::sleep(Duration::from_secs(2));
    let rss = process_rss_mb(child.id());

    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_file(&trace);

    match first_frame {
        Some(ms) => {
            println!("startup        first frame {ms} ms (wall {wall} ms, incl. file handshake)")
        }
        None => println!("startup        no trace produced (wall {wall} ms)"),
    }
    match rss {
        Some(mb) => println!("memory         {mb:.1} MB working set, idle"),
        None => println!("memory         unavailable"),
    }
}

fn process_rss_mb(pid: u32) -> Option<f64> {
    let script = format!(
        "$p=Get-Process -Id {pid} -ErrorAction SilentlyContinue; if($p){{[int64]$p.WorkingSet64}}"
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let bytes: u64 = text.trim().parse().ok()?;
    Some(bytes as f64 / (1024.0 * 1024.0))
}

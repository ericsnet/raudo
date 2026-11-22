//! Slint front-end wiring: navigation and directory loading.
//!
//! Directory reads happen on a worker thread and are pushed into the model
//! through the Slint event loop, so the window is never blocked by disk I/O.

slint::include_modules!();

pub use slint::PlatformError;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use raudo_core::{Entry, list_dir};
use slint::{Model, ModelRc, VecModel};

impl Row {
    fn from_entry(entry: &Entry) -> Self {
        Self {
            name: entry.name.as_str().into(),
            meta: if entry.is_dir {
                String::new()
            } else {
                raudo_core::format_size(entry.size)
            }
            .into(),
            is_dir: entry.is_dir,
        }
    }
}

/// Shows the window and runs the event loop until it is closed.
///
/// When `trace` points to a file, the time from process start to the first
/// visible frame is written there as `first_frame_ms=<n>`.
pub fn run(
    start: PathBuf,
    trace: Option<PathBuf>,
    started: Instant,
) -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let current = Rc::new(RefCell::new(start.clone()));

    load(&ui, start);

    {
        let ui_weak = ui.as_weak();
        let current = Rc::clone(&current);
        ui.on_open_row(move |index| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let Some(row) = ui.get_rows().row_data(index as usize) else {
                return;
            };
            if !row.is_dir {
                // Opening files arrives in Fase 2.
                return;
            }
            let next = current.borrow().join(row.name.as_str());
            *current.borrow_mut() = next.clone();
            load(&ui, next);
        });
    }

    {
        let ui_weak = ui.as_weak();
        let current = Rc::clone(&current);
        ui.on_go_up(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let parent = current.borrow().parent().map(Path::to_path_buf);
            let Some(parent) = parent else {
                return;
            };
            *current.borrow_mut() = parent.clone();
            load(&ui, parent);
        });
    }

    ui.show()?;

    if let Some(path) = trace {
        let _ = std::fs::write(
            path,
            format!("first_frame_ms={}\n", started.elapsed().as_millis()),
        );
    }

    slint::run_event_loop()?;
    Ok(())
}

fn load(ui: &AppWindow, dir: PathBuf) {
    ui.set_current_path(display(&dir).into());

    let weak = ui.as_weak();
    let worker_dir = dir.clone();
    std::thread::spawn(move || {
        let result = list_dir(&worker_dir)
            .map(|entries| entries.iter().map(Row::from_entry).collect::<Vec<Row>>());

        let _ = slint::invoke_from_event_loop(move || {
            let Some(ui) = weak.upgrade() else {
                return;
            };
            match result {
                Ok(rows) => {
                    ui.set_rows(ModelRc::new(VecModel::from(rows)));
                }
                Err(err) => {
                    ui.set_rows(ModelRc::new(VecModel::from(Vec::<Row>::new())));
                    ui.set_current_path(format!("{} — {}", display(&worker_dir), err).into());
                }
            }
        });
    });
}

fn display(path: &Path) -> String {
    path.display().to_string()
}

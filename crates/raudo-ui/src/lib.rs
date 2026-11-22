//! Slint front-end: tabs, history, breadcrumb, sorting and directory loading.
//!
//! Directory reads happen on a worker thread and are pushed into the model
//! through the Slint event loop, so the window is never blocked by disk I/O.

slint::include_modules!();

pub use slint::PlatformError;

use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use raudo_core::{Entry, SortDir, SortKey, drives, format_size, list_dir, sort_entries};
use slint::{Model, ModelRc, VecModel};

struct TabState {
    path: Option<PathBuf>,
    back: Vec<Option<PathBuf>>,
    forward: Vec<Option<PathBuf>>,
    entries: Vec<Entry>,
    sort_key: SortKey,
    sort_dir: SortDir,
    generation: u64,
}

impl TabState {
    fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            back: Vec::new(),
            forward: Vec::new(),
            entries: Vec::new(),
            sort_key: SortKey::Name,
            sort_dir: SortDir::Asc,
            generation: 0,
        }
    }
}

struct State {
    tabs: Vec<TabState>,
    active: usize,
}

impl State {
    fn active_tab(&self) -> &TabState {
        &self.tabs[self.active]
    }

    fn active_tab_mut(&mut self) -> &mut TabState {
        let active = self.active;
        &mut self.tabs[active]
    }
}

enum NavMode {
    Push,
    Keep,
}

type Shared = Arc<Mutex<State>>;

pub fn run(
    start: Option<PathBuf>,
    trace: Option<PathBuf>,
    started: Instant,
) -> Result<(), PlatformError> {
    let ui = AppWindow::new()?;
    let state: Shared = Arc::new(Mutex::new(State {
        tabs: vec![TabState::new(start.clone())],
        active: 0,
    }));

    ui.set_drives(ModelRc::new(VecModel::from(
        drives().iter().map(Drive::from_info).collect::<Vec<_>>(),
    )));
    ui.set_pinned(ModelRc::new(VecModel::from(pinned_items())));
    if let Some(image) = background_image() {
        ui.set_background_image(image);
    }

    wire_callbacks(&ui, &state);
    navigate(&ui, &state, 0, start, NavMode::Keep);

    ui.show()?;

    // The window auto-sizes to the layout; request the preferred size once the
    // event loop is running so the value is not overwritten by the layout pass.
    let weak = ui.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(20), move || {
        if let Some(ui) = weak.upgrade() {
            ui.window()
                .set_size(slint::WindowSize::Logical(slint::LogicalSize::new(
                    1280.0, 800.0,
                )));
        }
    });

    if let Some(path) = trace {
        let _ = std::fs::write(
            path,
            format!("first_frame_ms={}\n", started.elapsed().as_millis()),
        );
    }

    slint::run_event_loop()?;
    Ok(())
}

impl Drive {
    fn from_info(info: &raudo_core::DriveInfo) -> Self {
        let usage = if info.total == 0 {
            0.0
        } else {
            (1.0 - info.free as f64 / info.total as f64) as f32
        };
        Self {
            letter: info.letter.as_str().into(),
            label: info.label.as_str().into(),
            detail: format!(
                "{} libres de {}",
                format_size(info.free),
                format_size(info.total)
            )
            .into(),
            usage,
            root: info.root.as_str().into(),
        }
    }
}

fn active_index(state: &Shared) -> usize {
    state.lock().unwrap().active
}

fn wire_callbacks(ui: &AppWindow, state: &Shared) {
    {
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        ui.on_select_tab(move |index| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            {
                let mut s = state.lock().unwrap();
                let index = index as usize;
                if index < s.tabs.len() {
                    s.active = index;
                }
            }
            render(&ui, &state.lock().unwrap());
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        ui.on_close_tab(move |index| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            {
                let mut s = state.lock().unwrap();
                let index = index as usize;
                if s.tabs.len() > 1 && index < s.tabs.len() {
                    s.tabs.remove(index);
                    if s.active >= s.tabs.len() {
                        s.active = s.tabs.len() - 1;
                    } else if s.active > index {
                        s.active -= 1;
                    }
                }
            }
            render(&ui, &state.lock().unwrap());
        });
    }

    {
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        ui.on_new_tab(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let (index, target) = {
                let mut s = state.lock().unwrap();
                let path = s.active_tab().path.clone();
                let index = s.active + 1;
                s.tabs.insert(index, TabState::new(path.clone()));
                s.active = index;
                (index, path)
            };
            navigate(&ui, &state, index, target, NavMode::Keep);
        });
    }

    ui.on_go_back({
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let target = {
                let mut s = state.lock().unwrap();
                let tab = s.active_tab_mut();
                let Some(previous) = tab.back.pop() else {
                    return;
                };
                tab.forward.push(tab.path.clone());
                previous
            };
            let index = active_index(&state);
            navigate(&ui, &state, index, target, NavMode::Keep);
        }
    });

    ui.on_go_forward({
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let target = {
                let mut s = state.lock().unwrap();
                let tab = s.active_tab_mut();
                let Some(next) = tab.forward.pop() else {
                    return;
                };
                tab.back.push(tab.path.clone());
                next
            };
            let index = active_index(&state);
            navigate(&ui, &state, index, target, NavMode::Keep);
        }
    });

    ui.on_go_up({
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let target = {
                let s = state.lock().unwrap();
                s.active_tab()
                    .path
                    .as_deref()
                    .and_then(Path::parent)
                    .map(Path::to_path_buf)
            };
            let index = active_index(&state);
            navigate(&ui, &state, index, target, NavMode::Push);
        }
    });

    ui.on_go_home({
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let index = active_index(&state);
            navigate(&ui, &state, index, None, NavMode::Push);
        }
    });

    ui.on_refresh({
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let (index, target) = {
                let s = state.lock().unwrap();
                (s.active, s.active_tab().path.clone())
            };
            navigate(&ui, &state, index, target, NavMode::Keep);
        }
    });

    ui.on_open_crumb({
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        move |value| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let index = active_index(&state);
            navigate(
                &ui,
                &state,
                index,
                Some(PathBuf::from(value.as_str())),
                NavMode::Push,
            );
        }
    });

    ui.on_open_drive({
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        move |root| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let index = active_index(&state);
            navigate(
                &ui,
                &state,
                index,
                Some(PathBuf::from(root.as_str())),
                NavMode::Push,
            );
        }
    });

    ui.on_select_row({
        let ui_weak = ui.as_weak();
        move |index| {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_selected_index(index);
            }
        }
    });

    ui.on_toggle_fullscreen({
        let ui_weak = ui.as_weak();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                let window = ui.window();
                window.set_fullscreen(!window.is_fullscreen());
            }
        }
    });

    ui.on_open_row({
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        move |index| {
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
            let (tab_index, base) = {
                let s = state.lock().unwrap();
                (s.active, s.active_tab().path.clone())
            };
            let Some(base) = base else {
                return;
            };
            navigate(
                &ui,
                &state,
                tab_index,
                Some(base.join(row.name.as_str())),
                NavMode::Push,
            );
        }
    });

    ui.on_sort_by({
        let ui_weak = ui.as_weak();
        let state = Arc::clone(state);
        move |key| {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            let key = match key {
                1 => SortKey::Size,
                2 => SortKey::Modified,
                _ => SortKey::Name,
            };
            {
                let mut s = state.lock().unwrap();
                let tab = s.active_tab_mut();
                if tab.sort_key == key {
                    tab.sort_dir = match tab.sort_dir {
                        SortDir::Asc => SortDir::Desc,
                        SortDir::Desc => SortDir::Asc,
                    };
                } else {
                    tab.sort_key = key;
                    tab.sort_dir = SortDir::Asc;
                }
                let (key, dir) = (tab.sort_key, tab.sort_dir);
                sort_entries(&mut tab.entries, key, dir);
            }
            render(&ui, &state.lock().unwrap());
        }
    });
}

fn navigate(ui: &AppWindow, state: &Shared, index: usize, target: Option<PathBuf>, mode: NavMode) {
    let generation = {
        let mut s = state.lock().unwrap();
        let Some(tab) = s.tabs.get_mut(index) else {
            return;
        };
        if matches!(mode, NavMode::Push) {
            tab.back.push(tab.path.clone());
            tab.forward.clear();
        }
        tab.path = target.clone();
        tab.entries.clear();
        tab.generation += 1;
        tab.generation
    };

    render(ui, &state.lock().unwrap());

    let Some(dir) = target else {
        return;
    };
    let weak = ui.as_weak();
    let st = Arc::clone(state);
    std::thread::spawn(move || {
        let result = list_dir(&dir);
        let _ = slint::invoke_from_event_loop(move || {
            let Some(ui) = weak.upgrade() else {
                return;
            };
            {
                let mut s = st.lock().unwrap();
                if let Some(tab) = s.tabs.get_mut(index)
                    && tab.generation == generation
                {
                    match result {
                        Ok(mut entries) => {
                            sort_entries(&mut entries, tab.sort_key, tab.sort_dir);
                            tab.entries = entries;
                        }
                        Err(_) => tab.entries.clear(),
                    }
                }
            }
            render(&ui, &st.lock().unwrap());
        });
    });
}

fn render(ui: &AppWindow, state: &State) {
    let tab = state.active_tab();

    ui.set_tabs(ModelRc::new(VecModel::from(
        state
            .tabs
            .iter()
            .map(|t| TabItem {
                title: tab_title(t.path.as_deref()).into(),
            })
            .collect::<Vec<_>>(),
    )));
    ui.set_active_tab(state.active as i32);
    ui.set_is_home(tab.path.is_none());
    ui.set_can_back(!tab.back.is_empty());
    ui.set_can_forward(!tab.forward.is_empty());
    ui.set_crumbs(ModelRc::new(VecModel::from(
        tab.path.as_deref().map(crumbs).unwrap_or_default(),
    )));
    ui.set_current_path(
        tab.path
            .as_deref()
            .map(path_text)
            .unwrap_or_else(|| "Inicio".to_string())
            .into(),
    );
    ui.set_rows(ModelRc::new(VecModel::from(
        tab.entries
            .iter()
            .map(|entry| Row {
                name: entry.name.as_str().into(),
                meta: if entry.is_dir {
                    String::new()
                } else {
                    format_size(entry.size)
                }
                .into(),
                date: entry.modified_text.as_str().into(),
                is_dir: entry.is_dir,
            })
            .collect::<Vec<_>>(),
    )));
    ui.set_item_count(tab.entries.len() as i32);
    ui.set_selected_index(-1);
    ui.set_sort_key(tab.sort_key as i32);
    ui.set_sort_dir(if tab.sort_dir == SortDir::Asc { 0 } else { 1 });
}

fn tab_title(path: Option<&Path>) -> String {
    let Some(path) = path else {
        return "Inicio".to_string();
    };
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .or_else(|| {
            path.components().find_map(|c| match c {
                Component::Prefix(prefix) => {
                    Some(prefix.as_os_str().to_string_lossy().into_owned())
                }
                _ => None,
            })
        })
        .unwrap_or_else(|| path_text(path))
}

fn path_text(path: &Path) -> String {
    path.display().to_string()
}

fn crumbs(path: &Path) -> Vec<Crumb> {
    let mut result = Vec::new();
    let mut acc = PathBuf::new();
    for component in path.components() {
        acc.push(component);
        let name = match component {
            Component::Prefix(prefix) => prefix.as_os_str().to_string_lossy().into_owned(),
            Component::Normal(name) => name.to_string_lossy().into_owned(),
            _ => continue,
        };
        result.push(Crumb {
            name: name.into(),
            value: path_text(&acc).into(),
        });
    }
    result
}

fn pinned_items() -> Vec<NavItem> {
    raudo_core::known_folders()
        .into_iter()
        .map(|folder| NavItem {
            label: folder.label.into(),
            path: path_text(&folder.path).into(),
        })
        .collect()
}

fn background_image() -> Option<slint::Image> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("RAUDO_BACKGROUND") {
        candidates.push(PathBuf::from(path));
    }
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let dir = PathBuf::from(appdata).join("Raudo");
        for name in ["background.png", "background.jpg", "background.jpeg"] {
            candidates.push(dir.join(name));
        }
    }
    candidates
        .into_iter()
        .find_map(|path| slint::Image::load_from_path(&path).ok())
}

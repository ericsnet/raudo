//! Core services for Raudo: fast Windows filesystem access, drive inventory,
//! and (later) the NTFS index used for instant search.

pub mod drives;
pub mod fs;
pub mod icons;
pub mod known_folders;
pub mod shell;
pub mod walk;
pub mod wallpaper;

pub use drives::{DriveInfo, drives};
pub use fs::{Entry, ListError, SortDir, SortKey, format_size, list_dir, set_hidden, sort_entries};
pub use icons::ShellIcon;
pub use known_folders::{KnownFolder, known_folders};
pub use walk::{Hit, RESULT_LIMIT, search};
pub use wallpaper::desktop_wallpaper;

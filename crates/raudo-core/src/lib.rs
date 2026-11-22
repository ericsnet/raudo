//! Core services for Raudo: fast Windows filesystem access, drive inventory,
//! and (later) the NTFS index used for instant search.

pub mod drives;
pub mod fs;
pub mod known_folders;
pub mod walk;

pub use drives::{DriveInfo, drives};
pub use fs::{Entry, ListError, SortDir, SortKey, format_size, list_dir, sort_entries};
pub use known_folders::{KnownFolder, known_folders};
pub use walk::{Hit, RESULT_LIMIT, search};

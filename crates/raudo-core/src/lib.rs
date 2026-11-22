//! Core services for Raudo: fast Windows filesystem access, and (later)
//! the NTFS index used for instant search.

pub mod fs;

pub use fs::{Entry, ListError, format_size, list_dir};

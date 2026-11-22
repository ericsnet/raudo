//! Recursive name search used by the search box.
//!
//! The walk is depth-first, skips reparse points (no loops), ignores hidden
//! directories by default, and reports results in batches so the UI can paint
//! them while the search is still running.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::fs::{Entry, list_dir};

/// Upper bound for a single search. Enough for real use, bounded memory.
pub const RESULT_LIMIT: usize = 2000;

/// One hit: the entry plus the folder it came from.
#[derive(Debug, Clone)]
pub struct Hit {
    pub folder: PathBuf,
    pub entry: Entry,
}

/// Searches `root` recursively for names containing `needle`.
///
/// `cancel` is checked between directories and entries; setting it stops the
/// walk at the next opportunity.
pub fn search(
    root: &Path,
    needle: &str,
    limit: usize,
    cancel: &AtomicBool,
    mut emit: impl FnMut(Vec<Hit>),
) {
    let needle = needle.trim().to_lowercase();
    if needle.is_empty() {
        return;
    }

    let mut stack = vec![root.to_path_buf()];
    let mut found = 0usize;

    while let Some(dir) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        let Ok(entries) = list_dir(&dir) else {
            continue;
        };

        let mut batch = Vec::new();
        for entry in &entries {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            if entry.name.to_lowercase().contains(&needle) {
                batch.push(Hit {
                    folder: dir.clone(),
                    entry: entry.clone(),
                });
                found += 1;
                if found >= limit {
                    break;
                }
            }
        }
        if !batch.is_empty() {
            emit(batch);
        }
        if found >= limit {
            return;
        }

        // Reverse order keeps the traversal alphabetical when popping.
        for entry in entries.iter().rev() {
            if entry.is_dir && !entry.is_reparse_point && !entry.is_hidden {
                stack.push(dir.join(&entry.name));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_nested_names() {
        let cancel = AtomicBool::new(false);
        let mut hits = Vec::new();
        search(
            Path::new("C:\\Windows\\System32"),
            "drivers",
            50,
            &cancel,
            |batch| hits.extend(batch),
        );
        assert!(!hits.is_empty(), "System32 has nested 'drivers' entries");
        assert!(
            hits.iter()
                .all(|h| h.entry.name.to_lowercase().contains("drivers"))
        );
    }

    #[test]
    fn empty_needle_returns_nothing() {
        let cancel = AtomicBool::new(false);
        let mut called = false;
        search(Path::new("C:\\Windows"), "  ", 10, &cancel, |_| {
            called = true;
        });
        assert!(!called);
    }
}

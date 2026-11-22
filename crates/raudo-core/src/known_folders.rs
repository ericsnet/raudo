//! Resolves the user's real known folders, honouring redirections such as
//! OneDrive-managed Desktop/Documents.

use std::path::PathBuf;

use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{
    FOLDERID_Desktop, FOLDERID_Documents, FOLDERID_Downloads, FOLDERID_Music, FOLDERID_Pictures,
    FOLDERID_Videos, KF_FLAG_DEFAULT, SHGetKnownFolderPath,
};

/// Label shown in the sidebar and the folder it points to.
#[derive(Debug, Clone)]
pub struct KnownFolder {
    pub label: String,
    pub path: PathBuf,
}

/// The standard user folders that exist on this machine, in sidebar order.
pub fn known_folders() -> Vec<KnownFolder> {
    let candidates: [(&str, &windows::core::GUID); 6] = [
        ("Escritorio", &FOLDERID_Desktop),
        ("Descargas", &FOLDERID_Downloads),
        ("Documentos", &FOLDERID_Documents),
        ("Imágenes", &FOLDERID_Pictures),
        ("Música", &FOLDERID_Music),
        ("Vídeos", &FOLDERID_Videos),
    ];

    candidates
        .iter()
        .filter_map(|(label, id)| {
            let path = resolve(id)?;
            Some(KnownFolder {
                label: (*label).to_string(),
                path,
            })
        })
        .collect()
}

fn resolve(id: &windows::core::GUID) -> Option<PathBuf> {
    // SAFETY: `id` points at a static GUID. The returned string is owned by
    // the shell allocator and freed with `CoTaskMemFree` below.
    let wide = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }.ok()?;

    let mut len = 0usize;
    // SAFETY: on success the API returns a NUL-terminated buffer.
    unsafe {
        while *wide.0.add(len) != 0 {
            len += 1;
        }
    }
    let text = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(wide.0, len) });
    // SAFETY: the buffer came from the shell allocator.
    unsafe { CoTaskMemFree(Some(wide.0.cast())) };

    let path = PathBuf::from(text);
    path.is_dir().then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_at_least_documents() {
        let folders = known_folders();
        assert!(!folders.is_empty());
        assert!(folders.iter().any(|f| f.label == "Documentos"));
        assert!(folders.iter().all(|f| f.path.is_dir()));
    }
}

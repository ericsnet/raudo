//! Real Windows shell icons, extracted once per type and cached in memory.
//!
//! Icons are rasterized from `HICON` into premultiplied RGBA so they can be
//! handed to the UI without extra conversion work per frame.

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, HGDIOBJ, SelectObject,
};
use windows::Win32::UI::Shell::{
    SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_USEFILEATTRIBUTES, SHGetFileInfoW,
};
use windows::Win32::UI::WindowsAndMessaging::{DI_NORMAL, DestroyIcon, DrawIconEx};
use windows::core::PCWSTR;

#[derive(Debug, Clone)]
pub struct ShellIcon {
    pub width: u32,
    pub height: u32,
    /// Premultiplied RGBA.
    pub pixels: Vec<u8>,
}

/// Icons are requested by key: `"dir"`, `"drive"` or a lowercase extension.
pub fn icon(key: &str) -> Option<ShellIcon> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<ShellIcon>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(hit) = cache.lock().unwrap().get(key).cloned() {
        return hit;
    }
    let extracted = extract(key);
    cache
        .lock()
        .unwrap()
        .insert(key.to_string(), extracted.clone());
    extracted
}

fn extract(key: &str) -> Option<ShellIcon> {
    let mut info = SHFILEINFOW::default();
    let flags = SHGFI_ICON | SHGFI_LARGEICON;
    let (path, attributes) = match key {
        "dir" => ("folder".to_string(), 0x0000_0010u32), // FILE_ATTRIBUTE_DIRECTORY
        "drive" => ("C:\\".to_string(), 0x0000_0080u32), // FILE_ATTRIBUTE_NORMAL
        extension => (format!("file.{extension}"), 0x0000_0080u32),
    };

    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let use_attributes = key != "drive";

    // SAFETY: `wide` is NUL-terminated, `info` is a valid writable struct and
    // its size is passed explicitly.
    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(attributes),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            flags
                | if use_attributes {
                    SHGFI_USEFILEATTRIBUTES
                } else {
                    Default::default()
                },
        )
    };
    if result == 0 || info.hIcon.is_invalid() {
        return None;
    }

    let icon = rasterize(info.hIcon);
    // SAFETY: the icon handle was returned by the shell and is ours to free.
    unsafe {
        let _ = DestroyIcon(info.hIcon);
    }
    icon
}

fn rasterize(hicon: windows::Win32::UI::WindowsAndMessaging::HICON) -> Option<ShellIcon> {
    const SIZE: i32 = 32;
    // SAFETY: every handle is checked before use and freed before returning.
    unsafe {
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() {
            return None;
        }

        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: SIZE,
                biHeight: -SIZE,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bits: *mut c_void = std::ptr::null_mut();
        let bitmap = match CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
            Ok(bitmap) => bitmap,
            Err(_) => {
                let _ = DeleteDC(dc);
                return None;
            }
        };
        if bits.is_null() {
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
            return None;
        }

        let previous: HGDIOBJ = SelectObject(dc, bitmap.into());
        let _ = DrawIconEx(dc, 0, 0, hicon, SIZE, SIZE, 0, None, DI_NORMAL);
        SelectObject(dc, previous);

        let count = (SIZE * SIZE * 4) as usize;
        let source = std::slice::from_raw_parts(bits as *const u8, count);
        let mut pixels = Vec::with_capacity(count);
        for index in (0..count).step_by(4) {
            let pixel = &source[index..index + 4];
            let (blue, green, red, alpha) = (pixel[0], pixel[1], pixel[2], pixel[3]);
            // Many shell icons come without an alpha channel: the colour is
            // valid but alpha is 0. Treat opaque colour as opaque.
            let alpha = if alpha == 0 && (red | green | blue) != 0 {
                255
            } else {
                alpha
            };
            let scale = alpha as u16;
            pixels.push((red as u16 * scale / 255) as u8);
            pixels.push((green as u16 * scale / 255) as u8);
            pixels.push((blue as u16 * scale / 255) as u8);
            pixels.push(alpha);
        }

        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(dc);
        let _ = HWND::default();

        Some(ShellIcon {
            width: SIZE as u32,
            height: SIZE as u32,
            pixels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_folder_and_extension_icons() {
        let folder = icon("dir").expect("folder icon");
        assert_eq!(folder.width, 32);
        assert_eq!(folder.pixels.len(), 32 * 32 * 4);
        assert!(icon("txt").is_some());
        assert!(icon("exe").is_some());

        let mut transparent = 0;
        let mut opaque = 0;
        let mut black_opaque = 0;
        let mut white_opaque = 0;
        for pixel in folder.pixels.chunks_exact(4) {
            if pixel[3] == 0 {
                transparent += 1;
            } else {
                opaque += 1;
                if pixel[0] < 8 && pixel[1] < 8 && pixel[2] < 8 {
                    black_opaque += 1;
                }
                if pixel[0] > 240 && pixel[1] > 240 && pixel[2] > 240 {
                    white_opaque += 1;
                }
            }
        }
        println!(
            "transparent={transparent} opaque={opaque} black={black_opaque} white={white_opaque}"
        );
        for key in ["txt", "exe", "png", "lnk"] {
            if let Some(ic) = icon(key) {
                let mut t = 0;
                let mut b = 0;
                for p in ic.pixels.chunks_exact(4) {
                    if p[3] == 0 {
                        t += 1
                    } else if p[0] < 8 && p[1] < 8 && p[2] < 8 {
                        b += 1
                    }
                }
                println!("{key}: transparent={t} black={b}");
            }
        }
        println!("corner={:?}", &folder.pixels[0..4]);
        assert!(transparent > 0, "folder icon must have transparent pixels");
    }
}

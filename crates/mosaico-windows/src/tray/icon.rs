//! The running and paused variants of the icon.

use mosaico_core::Result;
use mosaico_core::tray::paint_paused;
use windows::Win32::Foundation::TRUE;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateBitmap, DIB_RGB_COLORS, DeleteObject, GetDC,
    GetDIBits, HBITMAP, HGDIOBJ, ReleaseDC,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, DestroyIcon, GetIconInfo, GetSystemMetrics, HICON, ICONINFO,
    IDI_APPLICATION, IMAGE_ICON, LR_DEFAULTCOLOR, LoadIconW, LoadImageW, SM_CXSMICON,
};
use windows::core::PCWSTR;

use crate::error::OrPlatform;

/// The id of the application icon in the executable's resources.
const APP_ICON: u16 = 1;

/// The two icons: running, and paused, made from the running one.
pub(super) struct Icons {
    running: Icon,
    paused: Option<Icon>,
}

impl Icons {
    /// Loads the application icon at the size of the notification area,
    /// and makes the paused variant from it. Without the application
    /// icon, the system's default application icon is used, and paused
    /// looks the same as running.
    pub(super) fn load() -> Self {
        // SAFETY: GetSystemMetrics has no preconditions.
        let size = unsafe { GetSystemMetrics(SM_CXSMICON) }.max(16);
        let Some(running) = app_icon(size) else {
            return Self {
                running: default_icon(),
                paused: None,
            };
        };
        let paused = paused_variant(running.0, size)
            .inspect_err(|e| mosaico_core::log_warn!("No paused tray icon: {e}"))
            .ok();
        Self { running, paused }
    }

    /// The icon to show.
    pub(super) fn get(&self, paused: bool) -> HICON {
        match &self.paused {
            Some(icon) if paused => icon.0,
            _ => self.running.0,
        }
    }
}

/// An icon, destroyed on drop when it is ours to destroy.
struct Icon(HICON, bool);

impl Drop for Icon {
    fn drop(&mut self) {
        if self.1 {
            // SAFETY: The icon was made by this process and nothing uses
            // it once the tray window is gone.
            unsafe {
                let _ = DestroyIcon(self.0);
            }
        }
    }
}

fn app_icon(size: i32) -> Option<Icon> {
    // SAFETY: GetModuleHandleW(None) is the running executable, and
    // LoadImageW reads its icon resource, which may be absent.
    let handle = unsafe {
        let module = GetModuleHandleW(None).ok()?;
        let id = PCWSTR(usize::from(APP_ICON) as *const u16);
        LoadImageW(
            Some(module.into()),
            id,
            IMAGE_ICON,
            size,
            size,
            LR_DEFAULTCOLOR,
        )
        .ok()?
    };
    Some(Icon(HICON(handle.0), true))
}

fn default_icon() -> Icon {
    // SAFETY: A shared system icon, never destroyed.
    let icon = unsafe { LoadIconW(None, IDI_APPLICATION) }.unwrap_or_default();
    Icon(icon, false)
}

fn paused_variant(icon: HICON, size: i32) -> Result<Icon> {
    let mut pixels = read_pixels(icon, size)?;
    paint_paused(&mut pixels, size as usize);
    make_icon(&pixels, size)
}

/// The pixels of `icon`, in RGBA rows top to bottom.
fn read_pixels(icon: HICON, size: i32) -> Result<Vec<[u8; 4]>> {
    let mut info = ICONINFO::default();
    // SAFETY: GetIconInfo copies the icon's bitmaps, deleted below.
    unsafe { GetIconInfo(icon, &mut info) }.or_platform("read the tray icon")?;
    let _bitmaps = Bitmaps([info.hbmColor, info.hbmMask]);
    let mut header = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -size,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bgra = vec![[0_u8; 4]; (size * size) as usize];
    // SAFETY: The buffer holds size rows of size 32-bit pixels, which is
    // what the header asks for. The screen DC is released right after.
    let lines = unsafe {
        let dc = GetDC(None);
        let lines = GetDIBits(
            dc,
            info.hbmColor,
            0,
            size as u32,
            Some(bgra.as_mut_ptr().cast()),
            &mut header,
            DIB_RGB_COLORS,
        );
        ReleaseDC(None, dc);
        lines
    };
    if lines != size {
        let reason = windows::core::Error::from_win32();
        return Err(mosaico_core::Error::platform("read the tray icon", reason));
    }
    let no_alpha = bgra.iter().all(|p| p[3] == 0);
    Ok(bgra
        .into_iter()
        .map(|[b, g, r, a]| [r, g, b, if no_alpha { 0xFF } else { a }])
        .collect())
}

/// An icon of `pixels`, RGBA rows top to bottom.
fn make_icon(pixels: &[[u8; 4]], size: i32) -> Result<Icon> {
    let bgra: Vec<[u8; 4]> = pixels.iter().map(|&[r, g, b, a]| [b, g, r, a]).collect();
    // Rows of a one-bit bitmap are padded to 16 bits.
    let mask_bits = vec![0_u8; ((size + 15) / 16 * 2 * size) as usize];
    // SAFETY: Both buffers hold size rows of the bitmap they describe and
    // are copied into the bitmaps, which are deleted below.
    let bitmaps = unsafe {
        Bitmaps([
            CreateBitmap(size, size, 1, 32, Some(bgra.as_ptr().cast())),
            CreateBitmap(size, size, 1, 1, Some(mask_bits.as_ptr().cast())),
        ])
    };
    let [color, mask] = bitmaps.0;
    let info = ICONINFO {
        fIcon: TRUE,
        hbmMask: mask,
        hbmColor: color,
        ..Default::default()
    };
    // SAFETY: CreateIconIndirect copies both bitmaps into a new icon.
    let icon = unsafe { CreateIconIndirect(&info) }.or_platform("make the paused tray icon")?;
    Ok(Icon(icon, true))
}

/// Bitmaps deleted on drop.
struct Bitmaps([HBITMAP; 2]);

impl Drop for Bitmaps {
    fn drop(&mut self) {
        for bitmap in self.0 {
            if !bitmap.is_invalid() {
                // SAFETY: Each bitmap is ours and used by nothing else.
                unsafe {
                    let _ = DeleteObject(HGDIOBJ(bitmap.0));
                }
            }
        }
    }
}

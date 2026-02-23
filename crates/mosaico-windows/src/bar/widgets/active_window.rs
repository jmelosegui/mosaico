//! Active window icon widget for the status bar.
//!
//! Extracts the application icon via `SHGetFileInfoW` and renders it
//! inside a pill at the icon's native resolution to avoid scaling
//! distortion.

use std::ffi::OsStr;
use std::mem;
use std::os::windows::ffi::OsStrExt;

use mosaico_core::config::bar::BarConfig;
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetObjectW, SelectObject,
};
use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW};
use windows::Win32::UI::WindowsAndMessaging::{
    DI_NORMAL, DestroyIcon, DrawIconEx, GetIconInfo, GetWindowThreadProcessId, HICON, ICONINFO,
};
use windows::core::PCWSTR;

use super::super::text::{DrawCtx, draw_pill};

/// Draws the focused window's icon inside a pill. Returns the new X.
pub fn draw(ctx: &mut DrawCtx, x: i32, config: &BarConfig, hwnd: Option<usize>) -> i32 {
    let Some(raw) = hwnd.filter(|&h| h != 0) else {
        return x;
    };
    let Some(hicon) = extract_icon(raw) else {
        return x;
    };

    let pill_y = (ctx.h / 8).max(1);
    let pill_h = ctx.h - pill_y * 2;
    let pill_w = pill_h; // square pill

    draw_pill(
        ctx,
        x,
        pill_y,
        pill_w,
        pill_h,
        &config.colors.widget_background,
        config.pill_radius,
        &config.colors.pill_border,
        config.pill_border_width,
    );

    // Cap icon to ~60% of pill height for comfortable margin.
    let native = native_icon_size(hicon);
    let icon_sz = native.min(pill_h * 3 / 5).max(1);
    let ix = x + (pill_w - icon_sz) / 2;
    let iy = pill_y + (pill_h - icon_sz) / 2;
    render_icon(ctx, hicon, ix, iy, icon_sz);

    unsafe {
        let _ = DestroyIcon(hicon);
    }
    x + pill_w + config.item_gap
}

// -- icon extraction ------------------------------------------------------

/// Resolves HWND -> PID -> exe path -> shell icon.
fn extract_icon(hwnd: usize) -> Option<HICON> {
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(HWND(hwnd as *mut _), Some(&mut pid)) };
    if pid == 0 {
        return None;
    }
    shell_icon(&exe_path(pid)?)
}

/// Returns the executable path for a process ID.
fn exe_path(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let len = K32GetModuleFileNameExW(Some(h), None, &mut buf);
        let _ = CloseHandle(h);
        if len == 0 {
            return None;
        }
        String::from_utf16(&buf[..len as usize]).ok()
    }
}

/// Gets a large shell icon for the given executable path.
fn shell_icon(path: &str) -> Option<HICON> {
    let wide: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();
    let mut info: SHFILEINFOW = unsafe { mem::zeroed() };
    let ok = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            Default::default(),
            Some(&mut info),
            mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if ok != 0 && !info.hIcon.0.is_null() {
        Some(info.hIcon)
    } else {
        None
    }
}

/// Queries the HICON for its native pixel width. Falls back to 32.
fn native_icon_size(hicon: HICON) -> i32 {
    unsafe {
        let mut ii: ICONINFO = mem::zeroed();
        if GetIconInfo(hicon, &mut ii).is_err() {
            return 32;
        }
        let mut bm: BITMAP = mem::zeroed();
        let sz = if !ii.hbmColor.0.is_null() {
            let cb = GetObjectW(
                ii.hbmColor.into(),
                mem::size_of::<BITMAP>() as i32,
                Some(&mut bm as *mut BITMAP as *mut _),
            );
            if cb > 0 { bm.bmWidth } else { 32 }
        } else {
            32
        };
        if !ii.hbmColor.0.is_null() {
            let _ = DeleteObject(ii.hbmColor.into());
        }
        if !ii.hbmMask.0.is_null() {
            let _ = DeleteObject(ii.hbmMask.into());
        }
        sz
    }
}

// -- icon rendering -------------------------------------------------------

/// Renders an HICON into the bar buffer via DrawIconEx + temp DIB.
fn render_icon(ctx: &mut DrawCtx, hicon: HICON, x: i32, y: i32, size: i32) {
    unsafe {
        let dc = CreateCompatibleDC(None);
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let Ok(bmp) = CreateDIBSection(Some(dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0) else {
            let _ = DeleteDC(dc);
            return;
        };
        if bits.is_null() {
            let _ = DeleteObject(bmp.into());
            let _ = DeleteDC(dc);
            return;
        }
        let old = SelectObject(dc, bmp.into());
        let buf = std::slice::from_raw_parts_mut(bits as *mut u32, (size * size) as usize);
        buf.fill(0);

        let _ = DrawIconEx(dc, 0, 0, hicon, size, size, 0, None, DI_NORMAL);
        blit_icon_pixels(ctx, buf, x, y, size);

        SelectObject(dc, old);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(dc);
    }
}

/// Composites icon pixels from the temp DIB into the bar buffer.
fn blit_icon_pixels(ctx: &mut DrawCtx, src_buf: &[u32], x: i32, y: i32, size: i32) {
    for iy in 0..size {
        for ix in 0..size {
            let (dx, dy) = (x + ix, y + iy);
            if dx < 0 || dx >= ctx.w || dy < 0 || dy >= ctx.h {
                continue;
            }
            let src = src_buf[(iy * size + ix) as usize];
            let sa = (src >> 24) & 0xFF;
            let di = (dy * ctx.w + dx) as usize;
            if sa == 0 {
                if src & 0x00FF_FFFF != 0 {
                    ctx.buf[di] = src | 0xFF00_0000;
                }
            } else if sa == 255 {
                ctx.buf[di] = src;
            } else {
                let dst = ctx.buf[di];
                let ia = 255 - sa;
                let r = ((src >> 16) & 0xFF) + (((dst >> 16) & 0xFF) * ia / 255);
                let g = ((src >> 8) & 0xFF) + (((dst >> 8) & 0xFF) * ia / 255);
                let b = (src & 0xFF) + ((dst & 0xFF) * ia / 255);
                let a = sa + (((dst >> 24) & 0xFF) * ia / 255);
                ctx.buf[di] = (a << 24) | (r << 16) | (g << 8) | b;
            }
        }
    }
}

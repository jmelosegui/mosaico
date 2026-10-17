//! What the pointer grabs, as a real window answers the hit test: one of
//! the test's own with a caption and a sizing frame, never shown.
//!
//! - The test process is not DPI aware, so neither is a window it makes,
//!   which is asked from a thread that is per-monitor aware, as the
//!   daemon's are, at points in the display's own pixels. On a display
//!   that is not scaled those are the window's pixels too, nothing is
//!   mapped, and that test skips.
//! - A window a per-monitor aware thread makes is per-monitor aware too,
//!   so its pixels are the display's at any scale, and it is asked at them
//!   on every display, without mapping.

use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::HiDpi::LogicalToPhysicalPointForPerMonitorDPI;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, GetWindowRect, RegisterClassW, WNDCLASSW, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_OVERLAPPEDWINDOW,
};
use windows::core::{PCWSTR, w};

use crate::window::test_threads::{per_monitor_dpi_aware, within_a_second};
use crate::window::test_windows::test_wnd_proc;

use super::tests::edges;
use super::*;

/// A window with a caption and a sizing frame on the calling thread,
/// never shown, and destroyed when dropped.
struct Framed {
    hwnd: HWND,
    /// Its outer bounds, in its own pixels.
    outer: RECT,
    /// The top of its client area, in its own pixels.
    client_top: i32,
}

impl Framed {
    /// `None` on a machine with no desktop to create it on.
    fn new() -> Option<Self> {
        let class = w!("MosaicoFramedTestWindow");
        let (mut outer, mut client) = (RECT::default(), POINT::default());
        // SAFETY: registers the class, which fails harmlessly once it
        // exists, creates a window with valid arguments, and writes its
        // bounds and the origin of its client area on the stack.
        let hwnd = unsafe {
            let wc = WNDCLASSW {
                lpfnWndProc: Some(test_wnd_proc),
                lpszClassName: class,
                ..Default::default()
            };
            RegisterClassW(&wc);
            let (ex_style, style) = (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, WS_OVERLAPPEDWINDOW);
            let (x, y, null) = (200, 200, PCWSTR::null());
            let hwnd = CreateWindowExW(
                ex_style, class, null, style, x, y, 400, 300, None, None, None, None,
            )
            .ok()?;
            let _ = GetWindowRect(hwnd, &mut outer);
            let _ = ClientToScreen(hwnd, &mut client);
            hwnd
        };
        let client_top = client.y;
        Some(Self {
            hwnd,
            outer,
            client_top,
        })
    }

    /// `point`, in the window's own pixels, in the display's. Only a
    /// thread that is not DPI aware, as the window is not, finds it in the
    /// window: one that is takes it for a point in the display's pixels.
    fn physical(&self, (x, y): (i32, i32)) -> (i32, i32) {
        let mut point = POINT { x, y };
        // SAFETY: reads the DPI of a window this thread created, and
        // writes the point on the stack, which it leaves as it is when it
        // fails.
        let _ = unsafe { LogicalToPhysicalPointForPerMonitorDPI(Some(self.hwnd), &mut point) };
        (point.x, point.y)
    }
}

impl Drop for Framed {
    fn drop(&mut self) {
        // SAFETY: destroying a window this thread created.
        let _ = unsafe { DestroyWindow(self.hwnd) };
    }
}

#[test]
fn a_window_that_is_not_dpi_aware_says_what_its_caption_and_edges_grab() {
    // Arrange -- its caption, left edge, bottom right corner and client
    // area, each a few of its pixels in, in the display's pixels.
    let found = within_a_second(|| {
        let window = Framed::new()?;
        let (outer, top) = (window.outer, window.client_top);
        let (x, y) = (
            (outer.left + outer.right) / 2,
            (outer.top + outer.bottom) / 2,
        );
        let corner = (outer.right - 3, outer.bottom - 3);
        let points = [(x, top - 4), (outer.left + 2, y), corner, (x, y)];
        let physical = points.map(|point| window.physical(point));
        if physical == points {
            return None;
        }
        per_monitor_dpi_aware();

        // Act
        Some(physical.map(|at| hit_test(window.hwnd, at).and_then(grab_of)))
    });

    // Assert
    let Some(grabs) = found.expect("a hit test returned at once") else {
        return;
    };
    let (left, corner) = (
        edges(true, false, false, false),
        edges(false, false, true, true),
    );
    assert_eq!(grabs, [Some(Grab::TitleBar), left, corner, None]);
}

#[test]
fn a_window_in_the_display_s_own_pixels_says_what_its_caption_and_an_edge_grab() {
    // Arrange -- its caption and left edge, a few pixels in.
    let found = within_a_second(|| {
        per_monitor_dpi_aware();
        let window = Framed::new()?;
        let (outer, top) = (window.outer, window.client_top);
        let caption = ((outer.left + outer.right) / 2, top - 4);
        let edge = (outer.left + 2, (outer.top + outer.bottom) / 2);

        // Act
        Some([caption, edge].map(|at| hit_test(window.hwnd, at).and_then(grab_of)))
    });

    // Assert
    let Some(grabs) = found.expect("a hit test returned at once") else {
        return;
    };
    let left = edges(true, false, false, false);
    assert_eq!(grabs, [Some(Grab::TitleBar), left]);
}

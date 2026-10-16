//! The borders a placement posted to a maximized window of the test's
//! own is made with: those kept with its frames, matched to the DPI of its
//! display now.

use std::time::Instant;

use windows::Win32::UI::WindowsAndMessaging::{
    GWL_STYLE, GetWindowLongW, SetWindowLongW, WS_MAXIMIZE,
};

use crate::window::test_windows::HiddenWindow;

use super::super::frames::Kept;
use super::call_tests::{TARGET, first_frame};
use super::*;

/// The left, top, right and bottom borders.
fn sides(border: BorderOffset) -> [i32; 4] {
    [border.left, border.top, border.right, border.bottom]
}

/// Win32 windows as `Probe` reads and moves them, but for what is kept,
/// which is `0`.
struct Keeping(Kept);

impl FrameProbe for Keeping {
    type Kept = Kept;

    fn exists(&self, window: WindowId) -> bool {
        Probe.exists(window)
    }

    fn minimized(&self, window: WindowId) -> bool {
        Probe.minimized(window)
    }

    fn maximized(&self, window: WindowId) -> bool {
        Probe.maximized(window)
    }

    fn shown(&self, window: WindowId) -> bool {
        Probe.shown(window)
    }

    fn responding(&self, window: WindowId) -> bool {
        Probe.responding(window)
    }

    fn owner(&self, window: WindowId) -> Option<u64> {
        Probe.owner(window)
    }

    fn at(&self, window: WindowId) -> Option<Rect> {
        Probe.at(window)
    }

    fn keep(&self, _: WindowId) -> Option<Kept> {
        Some(self.0)
    }

    fn aim(&self, window: WindowId, kept: &Kept, rect: Rect) -> Rect {
        Probe.aim(window, kept, rect)
    }

    fn visible(&self, kept: &Kept, at: Rect) -> Rect {
        Probe.visible(kept, at)
    }

    fn post(&mut self, window: WindowId, to: Rect) -> bool {
        Probe.post(window, to)
    }
}

#[test]
fn a_placement_posted_to_a_maximized_window_uses_its_kept_borders_at_its_dpi_now() {
    // Arrange -- borders kept at a DPI other than its display's, of a
    // window of the test's thread, maximized after its frame.
    let Some(window) = HiddenWindow::new() else {
        return;
    };
    let win = Window::from_id(window.id());
    let border = BorderOffset {
        left: 7,
        top: 0,
        right: 7,
        bottom: 7,
    };
    let at = |dpi| Kept {
        border,
        scaled: false,
        dpi: Some(dpi),
    };
    let moved = |kept: &Kept| sides(kept.border_for(win)) != sides(border);
    let Some(kept) = [96, 192].map(at).into_iter().find(moved) else {
        return;
    };
    let control = Win32WindowControl::default();
    let frame = first_frame(window.id());
    let now = Instant::now();
    control
        .frames()
        .place(&[frame], &mut Keeping(kept), now, &mut Vec::new());
    // SAFETY: changes the style of a window this test owns.
    unsafe {
        let style = GetWindowLongW(window.hwnd(), GWL_STYLE);
        SetWindowLongW(window.hwnd(), GWL_STYLE, style | WS_MAXIMIZE.0 as i32);
    }

    // Act
    let placed = control.post_placement(window.id(), &TARGET);

    // Assert -- neither as kept nor as read now.
    let placed = placed.expect("the placement was posted");
    assert_eq!(sides(placed.border), sides(kept.border_for(win)));
}

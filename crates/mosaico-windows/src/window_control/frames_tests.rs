//! `place_frames` on windows of the test's own, which are never shown.
//! A window of the test's thread handles each frame as it is posted, and
//! the windows of other threads, in `frames_thread_tests.rs` and
//! `frames_order_tests.rs`, handle theirs late or never.
//!
//! Where a window got to is checked through what `place_frames` reports,
//! not through `Window::rect`: the test process is not DPI aware, as the
//! daemon is, so on a scaled display the DWM bounds `Window::rect` reads
//! are in other units than the ones windows are moved in.

use std::time::{Duration, Instant};

use mosaico_core::Rect;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_STYLE, GetWindowLongW, SetWindowLongW, WINDOW_STYLE, WS_MAXIMIZE, WS_MINIMIZE,
};

use super::*;
use crate::window::test_windows::HiddenWindow;
use crate::window_control::Win32WindowControl;

#[path = "frames_kept_tests.rs"]
mod kept_tests;
#[path = "frames_order_tests.rs"]
mod order_tests;
#[path = "frames_pace_tests.rs"]
mod pace_tests;
#[path = "frames_restore_tests.rs"]
mod restore_tests;
#[path = "frames_shown_tests.rs"]
mod shown_tests;
#[path = "frames_thread_tests.rs"]
mod thread_tests;

/// Where the frames take the windows.
const A: Rect = Rect {
    x: 40,
    y: 30,
    width: 120,
    height: 90,
};
const B: Rect = Rect {
    x: 60,
    y: 30,
    width: 120,
    height: 90,
};

pub(super) fn frame(window: WindowId, rect: Rect, first: bool) -> Frame {
    Frame {
        window,
        rect,
        first,
    }
}

/// Posts one frame and returns what was found of its window.
pub(super) fn post(control: &Win32WindowControl, frame: Frame) -> FrameOutcome {
    let mut found = Vec::new();
    control
        .place_frames(&[frame], &mut found)
        .expect("frames are supported");
    found[0]
}

/// What `act` returns, and how long it took.
pub(super) fn timed<T>(act: impl FnOnce() -> T) -> (T, Duration) {
    let started = Instant::now();
    let done = act();
    (done, started.elapsed())
}

/// The window's visible bounds now.
fn rect_of(window: WindowId) -> Option<Rect> {
    Window::from_id(window).rect().ok()
}

/// Whether the window has all of `style`.
fn has_style(window: &HiddenWindow, style: WINDOW_STYLE) -> bool {
    // SAFETY: a read-only query on a window this test owns.
    let current = unsafe { GetWindowLongW(window.hwnd(), GWL_STYLE) } as u32;
    current & style.0 == style.0
}

#[test]
fn a_window_that_handles_its_frames_is_moved_and_stays_hidden() {
    // Arrange
    let Some(window) = HiddenWindow::new() else {
        return;
    };
    let control = Win32WindowControl::default();
    let start = rect_of(window.id()).expect("the window exists");

    // Act -- the last frame again is not posted, and finds it there.
    let first = post(&control, frame(window.id(), A, true));
    let second = post(&control, frame(window.id(), B, false));
    let third = post(&control, frame(window.id(), B, false));

    // Assert
    assert_eq!(first, FrameOutcome::Handled { reached: start });
    assert_eq!(second, FrameOutcome::Handled { reached: A });
    assert_eq!(third, FrameOutcome::Handled { reached: B });
    assert!(!Window::from_id(window.id()).is_visible(), "it was shown");
}

#[test]
fn a_minimized_or_maximized_window_is_left_alone() {
    for (style, outcome) in [
        (WS_MINIMIZE, FrameOutcome::Minimized),
        (WS_MAXIMIZE, FrameOutcome::Maximized),
    ] {
        // Arrange -- only the state, on a window that stays hidden.
        let Some(window) = HiddenWindow::new() else {
            return;
        };
        // SAFETY: changes the style of a window this test owns.
        unsafe {
            let current = GetWindowLongW(window.hwnd(), GWL_STYLE);
            SetWindowLongW(window.hwnd(), GWL_STYLE, current | style.0 as i32);
        }
        let control = Win32WindowControl::default();
        let start = rect_of(window.id());

        // Act
        let found = post(&control, frame(window.id(), A, true));

        // Assert
        assert_eq!(found, outcome);
        assert_eq!(rect_of(window.id()), start, "{outcome:?} window moved");
        assert!(has_style(&window, style), "{outcome:?} state changed");
        assert!(!Window::from_id(window.id()).is_visible());
    }
}

#[test]
fn a_window_that_is_gone_is_reported_gone() {
    // Arrange
    let Some(window) = HiddenWindow::new() else {
        return;
    };
    let id = window.id();
    drop(window);

    // Act
    let found = post(&Win32WindowControl::default(), frame(id, A, true));

    // Assert
    assert_eq!(found, FrameOutcome::Gone);
}

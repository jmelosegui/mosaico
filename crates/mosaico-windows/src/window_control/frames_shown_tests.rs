//! What keeps its order with the frames posted to a window of the test's
//! own that counts as shown, but never appears: a child of a hidden
//! window, which may be maximized, restored and minimized all the same.

use mosaico_core::WindowControl;
use mosaico_core::config::HidingBehaviour;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_STYLE, GetWindowLongW, GetWindowPlacement, WINDOWPLACEMENT, WS_VISIBLE,
};

use crate::monitor::rect_from_win32;
use crate::window::test_threads::{ThreadWindow, within_a_second, within_two_seconds};

use super::*;

/// Where the windows are placed.
const TARGET: Rect = Rect {
    x: 200,
    y: 100,
    width: 300,
    height: 200,
};

/// Where the window goes back to once restored, in outer bounds.
fn restores_to(window: WindowId) -> Option<Rect> {
    let mut placement = WINDOWPLACEMENT {
        length: size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    // SAFETY: a read into a struct on the stack, of a window this test
    // owns.
    unsafe { GetWindowPlacement(Window::from_id(window).hwnd(), &mut placement) }.ok()?;
    Some(rect_from_win32(&placement.rcNormalPosition))
}

/// Whether the window has `WS_VISIBLE`, which a restore would give it.
fn has_visible_style(window: WindowId) -> bool {
    // SAFETY: a read-only query on a window this test owns.
    let style = unsafe { GetWindowLongW(Window::from_id(window).hwnd(), GWL_STYLE) } as u32;
    style & WS_VISIBLE.0 != 0
}

#[test]
fn a_maximized_window_placed_right_after_a_frame_is_restored_first_without_waiting() {
    // Arrange -- maximized while its frame is still queued, as the user
    // would, and in view.
    let Some(owned) = ThreadWindow::child(true) else {
        return;
    };
    let window = owned.id();
    let control = Win32WindowControl::default();
    post(&control, frame(window, A, true));
    assert!(owned.maximize(), "the window was not maximized");

    // Act -- ending the maximized state at once would wait for the thread.
    let placed = within_a_second(move || control.place(window, TARGET).is_ok());
    owned.pump();
    let restored = || !Window::from_id(window).is_maximized();
    let restored_there = within_two_seconds(|| restored() && rect_of(window) == Some(TARGET));
    let end = rect_of(window);
    drop(owned);

    // Assert -- restored, then moved: a restore after the move would have
    // put it back where it was before it was maximized.
    assert_eq!(placed, Some(true));
    assert!(restored_there, "ended at {end:?}");
}

#[test]
fn a_maximized_window_out_of_view_is_placed_behind_a_frame_without_being_shown() {
    // Arrange -- without WS_VISIBLE, and with its thread getting to its
    // messages, as ending its maximized state then waits for it.
    let Some(owned) = ThreadWindow::child(false) else {
        return;
    };
    let window = owned.id();
    let control = Win32WindowControl::default();
    post(&control, frame(window, A, true));
    assert!(owned.maximize(), "the window was not maximized");
    owned.pump();

    // Act
    let placed = within_a_second(move || control.place(window, TARGET).is_ok());
    let placed_there = within_two_seconds(|| rect_of(window) == Some(TARGET));
    let end = rect_of(window);
    let (maximized, shown) = (
        Window::from_id(window).is_maximized(),
        has_visible_style(window),
    );
    drop(owned);

    // Assert -- not restored, which would have shown it.
    assert_eq!(placed, Some(true));
    assert!(placed_there, "ended at {end:?}");
    assert!(!maximized && !shown);
}

#[test]
fn a_minimize_made_right_after_a_frame_lands_after_it() {
    // Arrange -- in view, so it can be minimized, inside a hidden window.
    let Some(owned) = ThreadWindow::child(true) else {
        return;
    };
    let window = owned.id();
    let kept = Probe.keep(window).expect("the window exists");
    let framed = Probe.aim(window, &kept, A);
    let control = Win32WindowControl::default();
    post(&control, frame(window, A, true));

    // Act
    let how = HidingBehaviour::Minimize;
    let hidden = within_a_second(move || control.hide(window, how).is_ok());
    owned.pump();
    let minimized = within_two_seconds(|| Window::from_id(window).is_minimized());
    let restores = restores_to(window);
    drop(owned);

    // Assert -- it comes back where the frame took it. Had the frame
    // landed after, it would have moved the minimized window instead.
    assert_eq!(hidden, Some(true));
    assert!(minimized);
    assert_eq!(restores, Some(framed));
}

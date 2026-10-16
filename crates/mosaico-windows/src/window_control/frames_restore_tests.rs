//! How a placement posted behind a change of show state ends a maximized
//! state, or leaves it, on a child of a hidden window of the test's own:
//! shown, maximized, minimized and restored, it never appears.

use mosaico_core::WindowControl;
use mosaico_core::config::HidingBehaviour;

use crate::window::test_threads::{ThreadWindow, within_a_second, within_two_seconds};

use super::*;

/// Where the windows are placed.
const TARGET: Rect = Rect {
    x: 200,
    y: 100,
    width: 300,
    height: 200,
};

/// Whether the window is in its normal state, at `TARGET`.
fn restored_at_target(window: WindowId) -> bool {
    let win = Window::from_id(window);
    !win.is_maximized() && !win.is_minimized() && rect_of(window) == Some(TARGET)
}

/// Shows window `owned`, hidden by minimizing, and places it at `TARGET`
/// right after, its thread parked, then lets the thread get to it, and
/// says whether both returned at once and whether the window ended
/// restored at the target.
fn show_and_place(owned: ThreadWindow, control: Win32WindowControl) -> (Option<bool>, bool) {
    let window = owned.id();
    let how = HidingBehaviour::Minimize;
    let posted = within_a_second(move || {
        control.show(window, how).is_ok() && control.place(window, TARGET).is_ok()
    });
    owned.pump();
    let restored = within_two_seconds(|| restored_at_target(window));
    (posted, restored)
}

#[test]
fn a_window_minimized_from_maximized_is_shown_and_placed_restored_at_the_target() {
    // Arrange -- a frame queued, then the window maximized and minimized
    // by its own thread, so a show brings it back maximized.
    let Some(owned) = ThreadWindow::child(true) else {
        return;
    };
    let control = Win32WindowControl::default();
    post(&control, frame(owned.id(), A, true));
    assert!(
        owned.maximize() && owned.minimize(),
        "not minimized from maximized"
    );

    // Act -- as a workspace switch with hiding by minimizing does.
    let (posted, restored) = show_and_place(owned, control);

    // Assert -- not left maximized at the target's size.
    assert_eq!(posted, Some(true));
    assert!(restored, "not restored at the target");
}

#[test]
fn a_window_minimized_from_its_normal_state_is_shown_and_placed_at_the_target() {
    // Arrange
    let Some(owned) = ThreadWindow::child(true) else {
        return;
    };
    let control = Win32WindowControl::default();
    post(&control, frame(owned.id(), A, true));
    assert!(owned.minimize(), "the window was not minimized");

    // Act
    let (posted, restored) = show_and_place(owned, control);

    // Assert
    assert_eq!(posted, Some(true));
    assert!(restored, "not restored at the target");
}

#[test]
fn a_maximized_window_hidden_right_after_a_frame_is_placed_and_never_shown_again() {
    // Arrange -- maximized while its frame is queued, then hidden behind
    // it, its thread parked.
    let Some(owned) = ThreadWindow::child(true) else {
        return;
    };
    let window = owned.id();
    let control = Win32WindowControl::default();
    post(&control, frame(window, A, true));
    assert!(owned.maximize(), "the window was not maximized");
    let hidden = control.hide(window, HidingBehaviour::Hide).is_ok();

    // Act -- its style is left alone while the hide is queued, so the
    // placement does not wait for the window's thread.
    let placed = within_a_second(move || control.place(window, TARGET).is_ok());
    owned.pump();
    let end = within_two_seconds(|| rect_of(window) == Some(TARGET));
    let (maximized, shown) = (
        Window::from_id(window).is_maximized(),
        Window::from_id(window).is_shown(),
    );
    drop(owned);

    // Assert -- a restore behind the hide would have shown it again, and
    // so could its style cleared while the hide was on its way.
    assert!(hidden && end);
    assert_eq!(placed, Some(true));
    assert!(maximized && !shown);
}

#[test]
fn a_reveal_posted_notes_whether_it_leaves_the_window_in_view() {
    // Arrange -- two windows given a frame, one of them minimized, their
    // threads parked, so nothing they are posted lands.
    let (Some(shown), Some(minimized)) = (ThreadWindow::child(true), ThreadWindow::child(true))
    else {
        return;
    };
    let control = Win32WindowControl::default();
    let frames = [frame(shown.id(), A, true), frame(minimized.id(), A, true)];
    control
        .place_frames(&frames, &mut Vec::new())
        .expect("frames are supported");
    assert!(minimized.minimize(), "the window was not minimized");

    // Act -- `SW_SHOW` leaves a minimized window minimized.
    let in_view = [&shown, &minimized].map(|owned| {
        let window = owned.id();
        let revealed = control.show_revealed(window, Window::from_id(window));
        revealed
            .ok()
            .and_then(|()| control.frames().in_view_once_landed(window, &Probe))
    });
    std::thread::sleep(Duration::from_millis(1));

    // Assert
    assert_eq!(in_view, [Some(true), Some(false)]);
}

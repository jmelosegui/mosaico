//! What keeps its order with the frames posted to a hidden window of the
//! test's own: a placement, and changes of show state and of the stacking
//! order, posted behind the frames while they may be queued, so they land
//! after them.

use std::thread;
use std::time::{Duration, Instant};

use mosaico_core::WindowControl;
use mosaico_core::config::HidingBehaviour;

use crate::window::test_threads::{ThreadWindow, per_monitor_dpi_aware, within_a_second};
use crate::window::test_windows::HiddenWindow;

use super::*;

/// Where window `window` is, as a per-monitor aware thread reads it, once
/// it has got there or two seconds have passed.
fn ends_at(window: WindowId, target: Rect) -> Option<Rect> {
    let read = thread::spawn(move || {
        per_monitor_dpi_aware();
        let until = Instant::now() + Duration::from_secs(2);
        loop {
            let at = Window::from_id(window).rect().ok();
            if at == Some(target) || Instant::now() > until {
                return at;
            }
            thread::sleep(Duration::from_millis(5));
        }
    });
    read.join().expect("the thread ran")
}

#[test]
fn a_placement_made_right_after_a_frame_lands_after_it() {
    // Arrange -- the window's thread gets to its messages when told. The
    // threads that move and read it are per-monitor aware, as the daemon
    // is, and the target is whole in the window's own pixels at any scale.
    let Some(owned) = ThreadWindow::new() else {
        return;
    };
    let window = owned.id();
    let target = Rect::new(210, 210, 420, 210);

    // Act -- placed while the frame is still queued.
    let placed = within_a_second(move || {
        per_monitor_dpi_aware();
        let control = Win32WindowControl::default();
        post(&control, frame(window, A, true));
        control.place(window, target).is_ok()
    });
    owned.pump();
    let end = ends_at(window, target);
    drop(owned);

    // Assert -- posted, since a placement made at once would have waited
    // for the thread, and so landed last.
    assert_eq!(placed, Some(true));
    assert_eq!(end, Some(target));
}

#[test]
fn changes_of_show_state_made_right_after_a_frame_are_posted_behind_it() {
    // Arrange -- the window's thread never gets to them, so it is never
    // minimized or shown.
    let Some(owned) = ThreadWindow::new() else {
        return;
    };
    let window = owned.id();

    // Act -- a show state set at once would wait for that thread.
    let changed = within_a_second(move || {
        let control = Win32WindowControl::default();
        post(&control, frame(window, A, true));
        let how = HidingBehaviour::Minimize;
        let hidden = control.hide(window, how).is_ok() && control.minimize(window).is_ok();
        hidden && control.show(window, how).is_ok()
    });
    drop(owned);

    // Assert
    assert_eq!(changed, Some(true));
}

#[test]
fn a_frame_after_a_placement_takes_the_window_from_where_it_was_placed() {
    // Arrange / Act -- on a thread of its own, made per-monitor aware as
    // the daemon is, whose window takes each move at once.
    let found = thread::spawn(|| {
        per_monitor_dpi_aware();
        let window = HiddenWindow::new()?;
        let control = Win32WindowControl::default();
        post(&control, frame(window.id(), A, true));
        control.place(window.id(), B).ok()?;
        let placed = post(&control, frame(window.id(), A, false));
        let back = post(&control, frame(window.id(), B, false));
        Some((placed, back))
    });

    // Assert
    let Some((placed, back)) = found.join().expect("the thread ran") else {
        return;
    };
    assert_eq!(placed, FrameOutcome::Handled { reached: B });
    assert_eq!(back, FrameOutcome::Handled { reached: A });
}

#[test]
fn putting_a_window_behind_another_right_after_a_frame_is_posted_behind_it() {
    // Arrange -- the window's thread never gets to it.
    let (Some(owned), Some(other)) = (ThreadWindow::new(), HiddenWindow::new()) else {
        return;
    };
    let (window, behind) = (owned.id(), other.id());
    let control = Win32WindowControl::default();
    post(&control, frame(window, A, true));

    // Act -- reordered at once, it would wait for that thread.
    let reordered = within_a_second(move || control.put_behind(window, behind).is_ok());
    drop(owned);

    // Assert
    assert_eq!(reordered, Some(true));
}

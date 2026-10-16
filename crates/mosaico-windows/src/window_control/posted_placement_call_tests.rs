//! What a placement posted to a window of the test's own reads of it and
//! decides: the facts fed to the decisions, which their own tests take as
//! given.

use mosaico_core::frames::{Frame, FrameControl};

use crate::window::test_threads::{ThreadWindow, within_a_second, within_two_seconds};

use super::*;

/// Where the windows are placed.
pub(super) const TARGET: Rect = Rect {
    x: 200,
    y: 100,
    width: 300,
    height: 200,
};

/// The first frame of a motion of `window`.
pub(super) fn first_frame(window: WindowId) -> Frame {
    Frame {
        window,
        rect: Rect::new(40, 30, 120, 90),
        first: true,
    }
}

#[test]
fn a_placement_posted_to_a_hung_maximized_window_out_of_view_leaves_it_so_at_once() {
    // Arrange -- given a frame, maximized, and its thread parked until
    // Windows calls it not responding.
    let Some(owned) = ThreadWindow::child(false) else {
        return;
    };
    let window = owned.id();
    let control = Win32WindowControl::default();
    control
        .place_frames(&[first_frame(window)], &mut Vec::new())
        .expect("frames are supported");
    assert!(owned.maximize(), "the window was not maximized");
    let hung = (0..4).any(|_| within_two_seconds(|| !Probe.responding(window)));
    if !hung {
        return;
    }

    // Act -- clearing its style would wait for it.
    let placed = within_a_second(move || control.post_placement(window, &TARGET).map(|p| p.first));
    let maximized = Window::from_id(window).is_maximized();
    drop(owned);

    // Assert
    assert!(matches!(placed, Some(Ok(First::Nothing))), "{placed:?}");
    assert!(maximized);
}

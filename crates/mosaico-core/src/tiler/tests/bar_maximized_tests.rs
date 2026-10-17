//! Which bars have a maximized or fullscreen window under them, which the
//! bar then lets the pointer through to.

use std::time::Duration;

use crate::Rect;
use crate::bar::BarState;
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::{animated, start, still};
use crate::tiler::tests::{Harness, MON_H, MON_W};
use crate::{Action, Direction};

/// Which monitors of `h` have a maximized window under their bar.
pub(super) fn marked(h: &Harness) -> Vec<bool> {
    let mut states: Vec<BarState> = h.tiler.bar_states("", false);
    h.tiler.mark_maximized(&mut states, &h.desktop);
    states.iter().map(|s| s.maximized).collect()
}

#[test]
fn tiled_windows_leave_every_bar_taking_the_pointer() {
    // Arrange
    let h = Harness::new(2, &[(1, 0), (2, 1)]);

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(marks, [false, false]);
}

#[test]
fn a_maximized_window_marks_the_bar_of_its_monitor_only() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.window(2).maximized = true;

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(marks, [false, true]);
}

#[test]
fn a_window_over_the_whole_work_area_counts_as_fullscreen() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.window(1).rect = Rect::new(0, 0, MON_W, MON_H);

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(marks, [true, false]);
}

#[test]
fn a_window_short_of_the_work_area_does_not_count() {
    // Arrange -- as tall as the monitor, one pixel short of its width.
    let mut h = Harness::new(1, &[(1, 0)]);
    h.window(1).rect = Rect::new(0, 0, MON_W - 1, MON_H);

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(marks, [false]);
}

#[test]
fn a_window_that_starts_below_the_top_of_the_work_area_does_not_count() {
    // Arrange -- a pixel down, however far past the bottom it reaches.
    let mut h = Harness::new(1, &[(1, 0)]);
    h.window(1).rect = Rect::new(0, 1, MON_W, MON_H + 100);

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(marks, [false]);
}

#[test]
fn a_maximized_window_that_is_minimized_or_hidden_does_not_count() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.window(1).maximized = true;
    h.window(1).minimized = true;
    h.window(2).maximized = true;
    h.window(2).visible = false;

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(marks, [false, false]);
}

#[test]
fn the_managers_bar_states_mark_a_maximized_window() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 1)]);
    let mut manager = start(&platform, None, &still());
    manager.handle_action(&Action::Focus(Direction::Right));
    platform.edit(2, |w| w.maximized = true);

    // Act
    let states = manager.bar_states("", false);

    // Assert
    let marks: Vec<bool> = states.iter().map(|s| s.maximized).collect();
    assert_eq!(marks, [false, true]);
}

#[test]
fn a_window_a_minimize_is_on_its_way_to_does_not_count() {
    // Arrange -- window 1, slow to handle its frames, has a frame of a
    // swap on its way when, maximized meanwhile, it is minimized.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &animated());
    platform.edit(1, |w| w.frame_delay = 3);
    manager.handle_action(&Action::Move(Direction::Right));
    manager.flush_pending();
    platform.advance(Duration::from_millis(16));
    manager.frame();
    platform.edit(1, |w| w.maximized = true);
    manager.handle_action(&Action::MinimizeFocused);

    // Act
    let states = manager.bar_states("", false);

    // Assert -- the desktop still has it shown, until the minimize lands.
    assert!(!platform.window(1).minimized);
    let marks: Vec<bool> = states.iter().map(|s| s.maximized).collect();
    assert_eq!(marks, [false]);
}

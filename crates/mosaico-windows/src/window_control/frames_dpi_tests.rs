//! Frames for windows that do not follow the display's scale, on hidden
//! windows of the test's own. The test process is not DPI aware, so
//! neither are the windows it creates; the threads that move them are
//! made per-monitor aware, as the daemon is.

use std::thread;
use std::time::{Duration, Instant};

use mosaico_core::frames::{FrameOutcome, FrameProbe, PENDING_LIMIT};
use mosaico_core::{Rect, WindowId};

use super::super::Probe;
use super::super::tests::{frame, post, timed};
use super::*;
use crate::window::test_threads::{ThreadWindow, per_monitor_dpi_aware, within_a_second};
use crate::window::test_windows::HiddenWindow;
use crate::window_control::Win32WindowControl;

/// The visible bounds of `window` moved along by `dx`.
fn along(window: WindowId, dx: i32) -> Option<Rect> {
    let at = Window::from_id(window).rect().ok()?;
    Some(Rect { x: at.x + dx, ..at })
}

/// A frame the window cannot take: a pixel or two off where it is, but
/// mapped to where it is.
fn untakeable(window: WindowId) -> Option<Rect> {
    let kept = Probe.keep(window)?;
    let at = Probe.at(window)?;
    let start = along(window, 0)?;
    let (x, y, w, h) = (start.x, start.y, start.width, start.height);
    let mut nudged = [1, -1, 2, -2].into_iter().flat_map(|d| {
        [
            Rect::new(x + d, y, w, h),
            Rect::new(x, y + d, w, h),
            Rect::new(x, y, w + d, h),
            Rect::new(x, y, w, h + d),
        ]
    });
    nudged.find(|&rect| Probe.aim(window, &kept, rect) == at)
}

#[test]
fn only_a_per_monitor_aware_thread_maps_the_moves_of_a_window_that_is_not() {
    // Arrange
    let Some(window) = HiddenWindow::new() else {
        return;
    };
    let id = window.id();

    // Act
    let unaware = scaled(Window::from_id(id));
    let aware = thread::spawn(move || {
        per_monitor_dpi_aware();
        let own = HiddenWindow::new().map(|own| scaled(Window::from_id(own.id())));
        (scaled(Window::from_id(id)), own)
    });

    // Assert -- not for its own windows, which are aware too.
    assert!(!unaware);
    assert_eq!(aware.join().ok(), Some((true, Some(false))));
}

#[test]
fn a_frame_the_window_cannot_take_is_not_posted_and_finds_it_handled() {
    // Arrange -- frames to it stay queued. A display where every frame
    // moves the window, as one not scaled up, skips the test.
    let Some(owned) = ThreadWindow::new() else {
        return;
    };
    let window = owned.id();

    // Act
    let found = within_a_second(move || {
        per_monitor_dpi_aware();
        let near = Some(untakeable(window)?);
        let (start, far) = (along(window, 0), along(window, 50));
        let control = Win32WindowControl::default();
        let steps = [(near, true), (near, false), (far, false), (far, false)];
        let posted = |(rect, first): (Option<Rect>, bool)| {
            rect.map(|rect| post(&control, frame(window, rect, first)))
        };
        Some((start, timed(|| steps.map(posted))))
    });
    drop(owned);

    // Assert -- only the far frame was posted, and holds the next back.
    let Some((start, (outcomes, took))) = found.expect("place_frames returned at once") else {
        return;
    };
    let handled = start.map(|reached| FrameOutcome::Handled { reached });
    assert_eq!(outcomes[..3], [handled; 3]);
    if took < PENDING_LIMIT {
        assert_eq!(outcomes[3], Some(FrameOutcome::Pending));
    }
}

#[test]
fn a_frame_takes_the_window_where_it_maps_to() {
    // Arrange
    let Some(owned) = ThreadWindow::new() else {
        return;
    };
    owned.pump();
    let window = owned.id();

    // Act -- frames a pixel apart, not all of which the window can take.
    let found = thread::spawn(move || {
        per_monitor_dpi_aware();
        let kept = Probe.keep(window).expect("the window exists");
        let control = Win32WindowControl::default();
        let mut reached = Vec::new();
        for dx in 10..16 {
            let rect = Rect::new(100 + dx, 100 + dx / 2, 300 + dx, 200 + dx);
            post(&control, frame(window, rect, dx == 10));
            let aimed = Probe.aim(window, &kept, rect);
            let until = Instant::now() + Duration::from_secs(2);
            while Probe.at(window) != Some(aimed) && Instant::now() < until {
                thread::sleep(Duration::from_millis(5));
            }
            reached.push((Probe.at(window), aimed));
        }
        reached
    });
    let found = found.join().expect("the thread ran");
    drop(owned);

    // Assert
    for (at, aimed) in found {
        assert_eq!(at, Some(aimed));
    }
}

#[test]
fn a_value_is_scaled_and_rounded_halves_away_from_zero() {
    // Arrange / Act / Assert -- at 200 percent, and at 150.
    assert_eq!(mul_div(105, 96, 192), 53);
    assert_eq!(mul_div(104, 96, 192), 52);
    assert_eq!(mul_div(-101, 96, 192), -51);
    assert_eq!(mul_div(53, 192, 96), 106);
    assert_eq!(mul_div(1, 96, 144), 1);
    assert_eq!(mul_div(1, 144, 96), 2);
}

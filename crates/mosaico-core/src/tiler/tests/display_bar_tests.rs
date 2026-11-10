//! The bar's space across a display change that keeps the monitors as
//! they were.

use super::*;
use crate::Rect;
use crate::id::wid;
use crate::tiler::Effect;
use crate::tiler::Input;
use crate::tiler::tests::{Harness, displays};

/// The height of the bar on both monitors.
const BARS: i32 = 64;

/// No bar on any monitor.
const NO_BARS: i32 = 0;

/// A bar `height` pixels tall on both monitors of `h`.
fn bars(h: &Harness, height: i32) -> Reach {
    Reach::bar_on(&h.tiler.places(), height, &[0, 1])
}

/// Two monitors with a window each, tiled with a bar `height` pixels
/// tall reserved.
fn tiled_with(height: i32) -> Harness {
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.step(Input::Reach(bars(&h, height)));
    h
}

/// The tiling area of each monitor.
fn tiling_areas(h: &Harness) -> Vec<Rect> {
    h.tiler.monitors().iter().map(|m| m.work_area()).collect()
}

#[test]
fn bars_lost_in_a_display_change_that_keeps_the_monitors_free_their_space() {
    // Arrange -- review of #91: after the change, no bar could be made
    // again on the same two monitors.
    let mut h = tiled_with(BARS);

    // Act
    h.step(Input::Displays {
        monitors: displays(2),
        reach: bars(&h, NO_BARS),
    });

    // Assert -- tiled as if there had never been a bar.
    let expected = tiled_with(NO_BARS);
    assert_eq!(tiling_areas(&h), tiling_areas(&expected));
    assert_eq!(h.rect_of(1), expected.rect_of(1));
}

#[test]
fn bars_made_in_a_display_change_that_keeps_the_monitors_get_their_space() {
    // Arrange -- review of #91: no bar could be made at start, and the
    // change made them.
    let mut h = tiled_with(NO_BARS);

    // Act
    h.step(Input::Displays {
        monitors: displays(2),
        reach: bars(&h, BARS),
    });

    // Assert -- tiled as if the bars had been there from the start.
    let expected = tiled_with(BARS);
    assert_eq!(tiling_areas(&h), tiling_areas(&expected));
    assert_eq!(h.rect_of(1), expected.rect_of(1));
}

#[test]
fn a_display_change_that_changes_nothing_moves_nothing() {
    // Arrange -- several notifications can describe the same layout.
    let mut h = tiled_with(BARS);

    // Act
    let effects = h
        .step(Input::Displays {
            monitors: displays(2),
            reach: bars(&h, BARS),
        })
        .to_vec();

    // Assert
    assert!(effects.is_empty(), "{effects:?}");
}

#[test]
fn a_bar_that_moves_on_one_monitor_in_a_display_change_leaves_the_other_alone() {
    // Arrange -- window 1 on monitor 0, window 2 on monitor 1, no bars.
    let mut h = tiled_with(NO_BARS);
    let only_second = Reach::bar_on(&h.tiler.places(), BARS, &[1]);

    // Act
    let effects = h
        .step(Input::Displays {
            monitors: displays(2),
            reach: only_second,
        })
        .to_vec();

    // Assert -- only the monitor whose space moved is laid out again.
    let placed = |id| {
        effects
            .iter()
            .any(|e| matches!(*e, Effect::Place { window, .. } if window == wid(id)))
    };
    assert!(placed(2), "{effects:?}");
    assert!(!placed(1), "{effects:?}");
}

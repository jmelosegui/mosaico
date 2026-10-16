//! The bar's space across a display change that keeps the monitors as
//! they were.

use super::*;
use crate::Rect;
use crate::tiler::Input;
use crate::tiler::tests::{Harness, displays};

/// A bar on both monitors.
fn bars() -> BarReservation {
    BarReservation {
        height: 64,
        monitors: vec![0, 1],
    }
}

/// No bar on any monitor.
fn no_bars() -> BarReservation {
    BarReservation::default()
}

/// Two monitors with a window each, tiled with `bar` reserved.
fn tiled_with(bar: BarReservation) -> Harness {
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.step(Input::Bar(bar));
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
    let mut h = tiled_with(bars());

    // Act
    h.step(Input::Displays {
        monitors: displays(2),
        bar: no_bars(),
    });

    // Assert -- tiled as if there had never been a bar.
    let expected = tiled_with(no_bars());
    assert_eq!(tiling_areas(&h), tiling_areas(&expected));
    assert_eq!(h.rect_of(1), expected.rect_of(1));
}

#[test]
fn bars_made_in_a_display_change_that_keeps_the_monitors_get_their_space() {
    // Arrange -- review of #91: no bar could be made at start, and the
    // change made them.
    let mut h = tiled_with(no_bars());

    // Act
    h.step(Input::Displays {
        monitors: displays(2),
        bar: bars(),
    });

    // Assert -- tiled as if the bars had been there from the start.
    let expected = tiled_with(bars());
    assert_eq!(tiling_areas(&h), tiling_areas(&expected));
    assert_eq!(h.rect_of(1), expected.rect_of(1));
}

#[test]
fn a_display_change_that_changes_nothing_moves_nothing() {
    // Arrange -- several notifications can describe the same layout.
    let mut h = tiled_with(bars());

    // Act
    let effects = h
        .step(Input::Displays {
            monitors: displays(2),
            bar: bars(),
        })
        .to_vec();

    // Assert
    assert!(effects.is_empty(), "{effects:?}");
}

//! The resize keys on the focused tiled window: they move a split it
//! borders by the step a floating window takes, and the tiles beside it
//! follow.

use crate::id::wid;
use crate::tiler::Timing;
use crate::tiler::tests::Harness;
use crate::{Action, LayoutKind, WindowEvent};

/// Windows 1 and 2 side by side on one monitor, `focused` with focus.
fn two(focused: u64) -> Harness {
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::Focused {
        window: wid(focused),
    });
    h
}

/// The step the resize keys take across the first monitor of `h`.
fn width_step(h: &Harness) -> i32 {
    h.tiler.monitors()[0].work_area().width / 20
}

#[test]
fn the_resize_keys_move_the_split_a_tile_borders_by_a_step() {
    // Arrange
    let mut h = two(1);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    let step = width_step(&h);

    // Act
    h.action(Action::GrowWidth);
    let (wider, narrower) = (h.rect_of(1), h.rect_of(2));
    h.action(Action::ShrinkWidth);

    // Assert -- window 2 gives up what window 1 takes, the split is
    // remembered, and a shrink gives it back.
    assert_eq!(wider.width, left.width + step);
    assert_eq!(narrower.x, right.x + step);
    assert_eq!(narrower.width, right.width - step);
    assert_eq!((h.rect_of(1), h.rect_of(2)), (left, right));
}

#[test]
fn a_burst_of_presses_is_coalesced_and_steps_from_the_splits() {
    // Arrange
    let mut h = two(1);
    let left = h.rect_of(1);

    // Act
    for _ in 0..3 {
        h.action(Action::GrowWidth);
    }

    // Assert
    assert_eq!(h.rect_of(1).width, left.width + 3 * width_step(&h));
    let placements = h.placements();
    assert!(!placements.is_empty());
    assert!(placements.iter().all(|p| p.2 == Timing::Coalesced));
}

#[test]
fn the_last_tile_of_a_row_grows_towards_the_others() {
    // Arrange
    let mut h = two(2);
    let right = h.rect_of(2);

    // Act
    h.action(Action::GrowWidth);

    // Assert -- its right edge stays at the edge of the monitor.
    let grown = h.rect_of(2);
    assert_eq!(grown.x, right.x - width_step(&h));
    assert_eq!(grown.x + grown.width, right.x + right.width);
}

#[test]
fn the_height_keys_trade_height_down_a_stack() {
    // Arrange -- window 2 above window 3 in the stack.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.action(Action::CycleLayout);
    h.event(WindowEvent::Focused { window: wid(2) });
    let (master, top, bottom) = (h.rect_of(1), h.rect_of(2), h.rect_of(3));

    // Act
    h.action(Action::GrowHeight);

    // Assert
    let step = h.tiler.monitors()[0].work_area().height / 20;
    assert_eq!(h.rect_of(1), master);
    assert_eq!(h.rect_of(2).height, top.height + step);
    assert_eq!(h.rect_of(3).height, bottom.height - step);
}

#[test]
fn the_resize_keys_stop_where_the_split_goes_no_further() {
    // Arrange -- grown as far as the split goes.
    let mut h = two(1);
    for _ in 0..20 {
        h.action(Action::GrowWidth);
    }

    // Act
    h.action(Action::GrowWidth);

    // Assert
    assert!(h.placements().is_empty());
}

#[test]
fn the_resize_keys_do_nothing_in_monocle_or_on_a_maximized_tile() {
    // Arrange
    let mut monocle = two(1);
    monocle.action(Action::ToggleMonocle);
    let mut maximized = two(1);
    maximized.window(1).maximized = true;

    // Act
    monocle.action(Action::GrowWidth);
    maximized.action(Action::GrowWidth);

    // Assert
    assert!(monocle.placements().is_empty());
    assert!(maximized.placements().is_empty());
    assert!(
        maximized
            .workspace(0, 0)
            .split_ratios()
            .ratio(LayoutKind::Bsp, 0)
            .is_none()
    );
}

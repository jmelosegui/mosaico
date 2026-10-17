//! The preview and the bars: a window dragged over a bar, the wheel
//! turned over one meanwhile, and the bars while a window is dragged.

use crate::Action;
use crate::action::Origin;
use crate::tiler::Effect;
use crate::tiler::tests::{Harness, MON_H, MON_W};

use super::super::support::{center, drop_at, grab, report_at, reserve_bar, shown, swapping};

#[test]
fn over_its_own_monitors_bar_the_preview_shows_its_own_slot() {
    // Arrange
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    reserve_bar(&mut h);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    grab(&mut h, 1);

    // Act -- the pointer over the bar, above window 2.
    let effects = report_at(&mut h, 1, (right.center_x(), 10));

    // Assert -- over no tile, as over a gap.
    assert_eq!(shown(&effects), Some(left));
}

#[test]
fn over_another_monitors_bar_the_preview_shows_where_the_drop_puts_the_window() {
    // Arrange
    let windows = [(1, 0), (2, 0), (3, 1), (4, 1)];
    let mut h = Harness::with_settings(swapping(), 2, &windows);
    reserve_bar(&mut h);
    let first = h.rect_of(3);
    grab(&mut h, 1);
    let to = (first.center_x(), 10);

    // Act -- the pointer over the second monitor's bar, above window 3.
    let effects = report_at(&mut h, 1, to);
    drop_at(&mut h, 1, to);

    // Assert -- the slot at the end of that monitor's workspace, before
    // no tile.
    let slot = shown(&effects).expect("a preview");
    assert_eq!(h.active(1), vec![3, 4, 1]);
    assert_eq!(h.rect_of(1), slot);
}

#[test]
fn a_dragged_window_reporting_where_it_is_leaves_the_bars_as_they_were() {
    // Arrange -- the second monitor has no windows.
    let mut h = Harness::with_settings(swapping(), 2, &[(1, 0), (2, 0)]);
    reserve_bar(&mut h);
    h.tiler.take_bars_changed();
    grab(&mut h, 1);

    // Act -- window 1 is dragged over it.
    report_at(&mut h, 1, (MON_W + MON_W / 2, MON_H / 2));
    let changed = h.tiler.take_bars_changed();
    let mut states = h.tiler.bar_states("", false);
    h.tiler.mark_maximized(&mut states, &h.desktop);

    // Assert -- its reports neither have the bars drawn again nor let the
    // pointer through them.
    assert!(!changed);
    assert!(states.iter().all(|state| !state.maximized), "{states:?}");
}

#[test]
fn a_wheel_switch_over_a_bar_hides_the_preview_for_the_rest_of_the_drag() {
    // Arrange -- window 1's preview shows over window 2.
    let mut h = Harness::with_settings(swapping(), 2, &[(1, 0), (2, 0), (3, 1)]);
    let right = h.rect_of(2);
    grab(&mut h, 1);
    report_at(&mut h, 1, center(right));

    // Act -- the wheel over the second monitor's bar switches it, then the
    // window goes on moving over window 2.
    let switched = h
        .action_from(Action::GoToWorkspaceNext, Origin::Bar(1))
        .to_vec();
    let moved = report_at(&mut h, 1, (right.x + 20, right.y + 30));

    // Assert -- as a key leaves it.
    assert_eq!(switched.first(), Some(&Effect::HidePreview));
    assert_eq!(shown(&moved), None);
    assert_eq!(h.desktop.preview, None);
}

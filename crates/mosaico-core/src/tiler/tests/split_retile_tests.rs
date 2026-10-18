//! What the user's retile does to the splits they moved: the workspace on
//! screen of every monitor forgets them, in all its layouts, so every
//! split is back at the configured ratio, while a workspace out of view
//! keeps its own.

use crate::tiler::tests::Harness;
use crate::{Action, LayoutKind, Rect};

use super::split_tests::{move_splits_on, rects};

/// Whether workspace `ws` of monitor `mon` of `h` follows the configured
/// ratio in every split of every layout.
fn as_configured(h: &Harness, mon: usize, ws: usize) -> bool {
    let ws = h.workspace(mon, ws);
    let kinds = [
        LayoutKind::Bsp,
        LayoutKind::VerticalStack,
        LayoutKind::ThreeColumn,
    ];
    let follows = |kind| (0..ws.len()).all(|split| ws.split_ratios().ratio(kind, split).is_none());
    kinds.into_iter().all(follows)
}

#[test]
fn a_retile_puts_every_split_back_at_the_configured_ratio() {
    // Arrange -- three quarters of the width, then a quarter of the height
    // of the rest.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    let configured = rects(&h, &[1, 2, 3]);
    move_splits_on(&mut h, 0, &[0.75, 0.25]);
    let moved = rects(&h, &[1, 2, 3]);

    // Act
    h.action(Action::Retile);

    // Assert -- where the user had them, and then back as configured.
    let expected = [
        Rect::new(8, 8, 1424, 1064),
        Rect::new(1440, 8, 472, 262),
        Rect::new(1440, 278, 472, 794),
    ];
    assert_eq!(moved, expected);
    assert_eq!(rects(&h, &[1, 2, 3]), configured);
    assert!(as_configured(&h, 0, 0));
}

#[test]
fn a_retile_puts_back_the_splits_of_every_monitor() {
    // Arrange -- a split moved on each of two monitors, focus on the first.
    let mut h = Harness::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);
    let configured = rects(&h, &[1, 2, 3, 4]);
    move_splits_on(&mut h, 0, &[0.75]);
    move_splits_on(&mut h, 1, &[0.25]);
    let moved = rects(&h, &[1, 2, 3, 4]);

    // Act
    h.action(Action::Retile);

    // Assert
    assert_ne!(moved, configured);
    assert_eq!(rects(&h, &[1, 2, 3, 4]), configured);
    assert!(as_configured(&h, 0, 0) && as_configured(&h, 1, 0));
}

#[test]
fn a_retile_leaves_the_splits_of_a_workspace_out_of_view() {
    // Arrange -- the split of workspace 1 moved, then workspace 2 on screen.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    move_splits_on(&mut h, 0, &[0.75]);
    h.action(Action::GoToWorkspace(2));

    // Act
    h.action(Action::Retile);

    // Assert -- workspace 1 still has it.
    let kept = h.workspace(0, 0).split_ratios().ratio(LayoutKind::Bsp, 0);
    assert_eq!(kept, Some(0.75));
}

#[test]
fn a_retile_puts_back_the_splits_of_the_layouts_off_screen_too() {
    // Arrange -- the stack's master moved, then around to BSP.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.action(Action::CycleLayout);
    let stack_as_configured = h.rect_of(1);
    move_splits_on(&mut h, 0, &[0.25]);
    h.action(Action::CycleLayout);
    h.action(Action::CycleLayout);

    // Act
    h.action(Action::Retile);

    // Assert -- the stack, on screen again, has its master as configured.
    assert!(as_configured(&h, 0, 0));
    h.action(Action::CycleLayout);
    assert_eq!(h.rect_of(1), stack_as_configured);
}

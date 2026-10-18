//! The drops a key, a command, the pointer on a bar or a display change
//! made mid-drag leave as before drops moved windows: the window goes back
//! to its slot.

use crate::action::Origin;
use crate::tiler::Input;
use crate::tiler::tests::{Harness, displays};
use crate::{Action, Direction};

use super::super::super::support::{drop_at, grab, move_to, swapping};

#[test]
fn a_key_pressed_while_a_window_is_dragged_leaves_its_drop_as_before() {
    // Arrange -- dragged over window 2, then a key moves focus there.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    let over = (right.center_x(), right.center_y());
    grab(&mut h, 1);
    move_to(&mut h, 1, over);
    h.action(Action::Focus(Direction::Right));

    // Act
    drop_at(&mut h, 1, over);

    // Assert -- back to its slot, as a drop the tiler did not follow.
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.rect_of(1), left);
}

#[test]
fn a_wheel_switch_over_a_bar_while_a_window_is_dragged_leaves_its_drop_as_before() {
    // Arrange -- dragged over window 2, then the wheel over the second
    // monitor's bar switches that monitor to its next workspace.
    let mut h = Harness::with_settings(swapping(), 2, &[(1, 0), (2, 0), (3, 1)]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    let over = (right.center_x(), right.center_y());
    grab(&mut h, 1);
    move_to(&mut h, 1, over);
    h.action_from(Action::GoToWorkspaceNext, Origin::Bar(1));

    // Act
    drop_at(&mut h, 1, over);

    // Assert -- the switch was made, and the drop is as a key leaves it.
    assert_eq!(h.tiler.monitors()[1].active_workspace(), 1);
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.rect_of(1), left);
}

#[test]
fn a_display_change_while_a_window_is_dragged_leaves_its_drop_as_before() {
    // Arrange -- dragged over window 2, then the displays are read again.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    let over = (right.center_x(), right.center_y());
    grab(&mut h, 1);
    move_to(&mut h, 1, over);
    h.step(Input::Displays {
        monitors: displays(1),
        bar: Default::default(),
    });

    // Act
    drop_at(&mut h, 1, over);

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.rect_of(1), left);
}

#[test]
fn a_display_change_noted_while_a_window_is_dragged_leaves_its_drop_as_before() {
    // Arrange -- dragged over window 2, then a display change is noted,
    // before the monitors are read again.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    let over = (right.center_x(), right.center_y());
    grab(&mut h, 1);
    move_to(&mut h, 1, over);
    h.step(Input::DisplayChanging);

    // Act
    drop_at(&mut h, 1, over);

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.rect_of(1), left);
}

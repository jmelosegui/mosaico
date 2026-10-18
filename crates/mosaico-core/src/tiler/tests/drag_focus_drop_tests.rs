//! Focus as the drag of a window ends, as the owner chose while testing:
//! the drop focuses the window through the platform when no report said
//! the system did and the platform was not asked to, and leaves focus
//! where a key moved it on.

use crate::id::wid;
use crate::tiler::Effect;
use crate::tiler::tests::Harness;
use crate::{Action, Direction, WindowEvent};

use super::focus_tests::grab_unreported;

#[test]
fn the_drop_focuses_a_window_no_report_said_took_focus() {
    // Arrange -- window 2 grabbed while window 1 has focus.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    grab_unreported(&mut h, 2);

    // Act
    let dropped = h.event(WindowEvent::moved(wid(2))).to_vec();

    // Assert -- the platform is asked to focus it, the pointer left where
    // the user let go.
    let focus = Effect::Focus {
        window: wid(2),
        move_cursor: false,
    };
    assert!(dropped.contains(&focus), "{dropped:?}");
}

#[test]
fn a_report_that_the_dragged_window_took_focus_ends_nothing_and_leaves_the_drop_nothing_to_do() {
    // Arrange -- window 2 grabbed while window 1 has focus.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    grab_unreported(&mut h, 2);

    // Act -- the system reports it focused, then it is dropped.
    h.event(WindowEvent::Focused { window: wid(2) });
    let dragged = h.tiler.mouse.drag.is_some();
    let dropped = h.event(WindowEvent::moved(wid(2))).to_vec();

    // Assert
    assert!(dragged);
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
    let focuses = |e: &Effect| matches!(e, Effect::Focus { .. });
    assert!(!dropped.iter().any(focuses), "{dropped:?}");
}

#[test]
fn a_key_that_focuses_the_dragged_window_leaves_the_drop_nothing_to_do() {
    // Arrange -- window 1 grabbed while window 2 has focus.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    grab_unreported(&mut h, 1);

    // Act -- a key moves focus to window 2 and back, then the drop.
    h.action(Action::Focus(Direction::Right));
    h.action(Action::Focus(Direction::Left));
    let dropped = h.event(WindowEvent::moved(wid(1))).to_vec();

    // Assert -- the key asked the platform already.
    let focuses = |e: &Effect| matches!(e, Effect::Focus { .. });
    assert!(!dropped.iter().any(focuses), "{dropped:?}");
}

#[test]
fn a_key_that_moves_focus_on_mid_drag_leaves_the_drop_nothing_to_give_back() {
    // Arrange -- window 1 grabbed while window 2 has focus.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    grab_unreported(&mut h, 1);

    // Act -- a key moves focus on to window 2, then the drop.
    h.action(Action::Focus(Direction::Right));
    let dropped = h.event(WindowEvent::moved(wid(1))).to_vec();

    // Assert -- focus stays where the key put it.
    let to_first = |e: &Effect| matches!(e, Effect::Focus { window, .. } if *window == wid(1));
    assert!(!dropped.iter().any(to_first), "{dropped:?}");
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
}

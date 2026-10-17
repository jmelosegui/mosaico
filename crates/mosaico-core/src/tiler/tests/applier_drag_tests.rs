//! What the tiling manager applies while the user drags a window: what
//! was held back for it, and the pointer.

use crate::id::wid;
use crate::tiler::tests::fake_platform::{Call, FakePlatform};
use crate::tiler::tests::manager_support::{start, still};
use crate::{Action, Direction, Rect, WindowEvent};

/// The pointer at the center of `rect`.
pub(super) fn centered_on(rect: Rect) -> Call {
    Call::MoveCursor(rect.center_x(), rect.center_y())
}

/// Whether `calls` move the pointer.
pub(super) fn moves_pointer(calls: &[Call]) -> bool {
    calls.iter().any(|c| matches!(c, Call::MoveCursor(..)))
}

#[test]
fn what_was_held_back_for_a_window_the_user_grabs_is_dropped() {
    // Arrange -- window 1 swaps with window 2, both moves held back while
    // input goes on.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &still());
    let left = platform.window(1).rect;
    manager.handle_action(&Action::Move(Direction::Right));
    platform.take_calls();

    // Act -- the user grabs window 1 before they are made.
    manager.handle_event(&WindowEvent::move_started(wid(1)));
    manager.flush_pending();

    // Assert -- window 2 still goes, and window 1 stays with the user.
    let calls = platform.take_calls();
    assert!(calls.contains(&Call::Place(wid(2), left)), "{calls:?}");
    let placed_1 = |c: &Call| matches!(c, Call::Place(w, _) if *w == wid(1));
    assert!(!calls.iter().any(placed_1), "{calls:?}");
    assert!(!moves_pointer(&calls), "{calls:?}");
}

#[test]
fn a_window_opening_during_a_drag_takes_focus_and_the_pointer_once_it_ends() {
    // Arrange -- the pointer follows focus, and the user drags window 1.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    platform.edit(3, |w| w.visible = false);
    let mut manager = start(&platform, None, &still());
    manager.flush_pending();
    manager.handle_event(&WindowEvent::move_started(wid(1)));
    platform.take_calls();

    // Act -- window 3 opens while it is dragged, then the drag ends.
    platform.edit(3, |w| w.visible = true);
    manager.handle_event(&WindowEvent::Created { window: wid(3) });
    manager.flush_pending();
    let dragging = platform.take_calls();
    manager.handle_event(&WindowEvent::moved(wid(1)));

    // Assert -- it is tiled and focused at once, and the pointer goes to it
    // only after the drop.
    assert!(dragging.contains(&Call::Focus(wid(3))), "{dragging:?}");
    assert!(!moves_pointer(&dragging), "{dragging:?}");
    let calls = platform.take_calls();
    let at = platform.window(3).rect;
    assert_eq!(calls.last(), Some(&centered_on(at)), "{calls:?}");
}

#[test]
fn the_pointer_is_left_alone_while_a_window_the_tiler_leaves_alone_is_dragged() {
    // Arrange -- the user drags a window the tiler does not manage.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (9, 0)]);
    platform.edit(9, |w| w.facts = None);
    let mut manager = start(&platform, None, &still());
    manager.flush_pending();
    manager.handle_event(&WindowEvent::move_started(wid(9)));
    platform.take_calls();

    // Act -- focus goes to window 2 from the keyboard meanwhile.
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.flush_pending();
    let dragging = platform.take_calls();
    manager.handle_event(&WindowEvent::moved(wid(9)));

    // Assert
    assert!(dragging.contains(&Call::Focus(wid(2))), "{dragging:?}");
    assert!(!moves_pointer(&dragging), "{dragging:?}");
    let right = platform.window(2).rect;
    assert!(platform.take_calls().contains(&centered_on(right)));
}

#[test]
fn a_drag_during_which_nothing_moved_focus_moves_no_pointer_as_it_ends() {
    // Arrange -- the near miss: nothing asks for the pointer meanwhile.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &still());
    manager.flush_pending();
    manager.handle_event(&WindowEvent::move_started(wid(2)));
    platform.take_calls();

    // Act
    manager.handle_event(&WindowEvent::moved(wid(2)));
    manager.flush_pending();

    // Assert
    let calls = platform.take_calls();
    assert!(!moves_pointer(&calls), "{calls:?}");
}

#[test]
fn a_dialog_of_the_window_the_tiler_gave_focus_taking_it_keeps_the_pointer_held() {
    // Arrange -- window 2 owns dialog 9, which the system gives focus in
    // its place as a key gives focus to window 2 while window 1 is dragged.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (9, 0)]);
    platform.edit(9, |w| {
        w.facts = None;
        w.owner = Some(wid(2));
    });
    let mut manager = start(&platform, None, &still());
    manager.flush_pending();
    manager.handle_event(&WindowEvent::move_started(wid(1)));
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.flush_pending();
    platform.take_calls();

    // Act -- the dialog reports focus, then the user drops window 1.
    manager.handle_event(&WindowEvent::Focused { window: wid(9) });
    let reported = platform.take_calls();
    manager.handle_event(&WindowEvent::moved(wid(1)));

    // Assert -- the pointer goes to window 2 only then.
    assert!(!moves_pointer(&reported), "{reported:?}");
    let right = platform.window(2).rect;
    assert!(platform.take_calls().contains(&centered_on(right)));
}

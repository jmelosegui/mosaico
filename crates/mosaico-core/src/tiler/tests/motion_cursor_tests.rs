//! Where the pointer goes when it follows a window that is moving, or
//! that the user moved, and where it goes without transitions.

use crate::id::wid;
use crate::tiler::tests::fake_desktop::Queued;
use crate::tiler::tests::fake_platform::Call;
use crate::tiler::tests::manager_support::still;
use crate::{Action, Direction, Rect, WindowEvent};

use super::{REFRESH, animated, settle, swap, two};

/// The pointer at the center of `rect`.
fn centered_on(rect: Rect) -> Call {
    Call::MoveCursor(rect.center_x(), rect.center_y())
}

#[test]
fn the_pointer_that_follows_a_swapped_window_goes_to_where_it_is_going() {
    // Arrange -- the pointer follows focus, and window 1 has it.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);

    // Act
    swap(&mut manager);

    // Assert -- not to where window 1 still is.
    let calls = platform.take_calls();
    assert!(calls.contains(&centered_on(right)), "{calls:?}");
    assert!(!calls.contains(&centered_on(left)), "{calls:?}");
}

#[test]
fn focusing_a_moving_window_puts_the_pointer_on_its_target() {
    // Arrange -- the windows are on their way, window 2 to the left.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    swap(&mut manager);
    platform.take_calls();

    // Act
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();

    // Assert
    let calls = platform.take_calls();
    assert!(calls.contains(&Call::Focus(wid(2))), "{calls:?}");
    assert!(calls.contains(&centered_on(left)), "{calls:?}");
    assert!(!calls.contains(&centered_on(right)), "{calls:?}");
}

#[test]
fn without_transitions_the_pointer_follows_focus_to_where_the_desktop_has_the_window() {
    // Arrange -- the windows swap at once; then window 2's application
    // moves it a little.
    let (platform, mut manager) = two(&still());
    swap(&mut manager);
    platform.edit(2, |w| w.rect.x += 40);
    let at = platform.window(2).rect;
    platform.take_calls();

    // Act -- focus goes to it, the pointer following.
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();

    // Assert -- as before transitions, to where the desktop has it.
    let calls = platform.take_calls();
    assert!(calls.contains(&centered_on(at)), "{calls:?}");
}

#[test]
fn the_pointer_follows_focus_to_a_dragged_window_once_it_is_dropped() {
    // Arrange -- right after a swap, the user drags window 2 a little.
    let (platform, mut manager) = two(&animated());
    let left = platform.window(1).rect;
    swap(&mut manager);
    settle(&platform, &mut manager);
    manager.handle_event(&WindowEvent::move_started(wid(2)));
    platform.edit(2, |w| w.rect.x += 100);
    platform.take_calls();

    // Act -- focus goes to it, the pointer following, and then the user
    // drops it.
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();
    let dragging = platform.take_calls();
    manager.handle_event(&WindowEvent::moved(wid(2)));

    // Assert -- the pointer stays with the user while they drag, then goes
    // to the slot the window goes back to, not to where they left it.
    let moves = |c: &Call| matches!(c, Call::MoveCursor(..));
    assert!(dragging.contains(&Call::Focus(wid(2))), "{dragging:?}");
    assert!(!dragging.iter().any(moves), "{dragging:?}");
    let calls = platform.take_calls();
    assert!(calls.contains(&centered_on(left)), "{calls:?}");
}

#[test]
fn the_pointer_follows_focus_to_where_the_frame_posted_takes_a_window_stopped_on_its_way() {
    // Arrange -- window 1's application takes two frames to get to each,
    // so the frame after the one posted to it is held back; then a display
    // change stops it on its way.
    let (platform, mut manager) = two(&animated());
    platform.edit(1, |w| w.frame_delay = 2);
    swap(&mut manager);
    for _ in 0..2 {
        platform.advance(REFRESH);
        manager.frame();
    }
    let Queued::Frame(posted) = platform.window(1).queued[0].0 else {
        panic!("a frame queued for window 1");
    };
    manager.display_changing();
    platform.take_calls();

    // Act -- focus goes to window 2 and back, the pointer following.
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.flush_pending();

    // Assert -- to where that frame takes it, not the one held back.
    let calls = platform.take_calls();
    assert!(calls.contains(&centered_on(posted)), "{calls:?}");
}

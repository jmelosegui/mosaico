//! Where the pointer goes when it follows a window that is moving.

use crate::id::wid;
use crate::tiler::tests::fake_platform::Call;
use crate::{Action, Direction, Rect};

use super::{animated, swap, two};

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

//! The pointer a drag holds, and the focus a click or the wheel on a bar
//! moves meanwhile.

use crate::action::Origin;
use crate::id::wid;
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::{start, still};
use crate::{Action, Direction, WindowEvent};

use super::tests::{centered_on, moves_pointer};

#[test]
fn a_wheel_switch_over_a_bar_during_a_drag_leaves_the_pointer_where_the_user_has_it() {
    // Arrange -- the pointer follows focus. While window 1 is dragged, a
    // key gives window 2 focus, which the pointer is to follow once the
    // drag ends.
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = start(&platform, None, &still());
    manager.flush_pending();
    manager.handle_event(&WindowEvent::move_started(wid(1)));
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.flush_pending();
    platform.take_calls();

    // Act -- the wheel over the second monitor's bar switches it, which
    // moves focus there first, then the user drops window 1.
    manager.handle_action_from(&Action::GoToWorkspaceNext, Origin::Bar(1));
    manager.flush_pending();
    manager.handle_event(&WindowEvent::moved(wid(1)));
    manager.flush_pending();

    // Assert -- focus went where the wheel asked, and the pointer stays.
    let calls = platform.take_calls();
    assert!(!moves_pointer(&calls), "{calls:?}");
    assert_ne!(manager.tiler().focused_window(), Some(wid(2)));
}

#[test]
fn a_key_after_a_wheel_switch_during_a_drag_has_the_pointer_follow_it() {
    // Arrange -- the near miss: the key comes last.
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = start(&platform, None, &still());
    manager.flush_pending();
    manager.handle_event(&WindowEvent::move_started(wid(1)));
    manager.handle_action_from(&Action::GoToWorkspaceNext, Origin::Bar(1));
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();
    platform.take_calls();

    // Act
    manager.handle_event(&WindowEvent::moved(wid(1)));
    manager.flush_pending();

    // Assert -- the pointer goes to the window the key gave focus.
    let focused = manager
        .tiler()
        .focused_window()
        .expect("a window has focus");
    let at = platform.window(focused.raw()).rect;
    assert!(platform.take_calls().contains(&centered_on(at)));
}

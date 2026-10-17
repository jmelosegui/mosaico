//! The pointer a drag holds through the focus changes the tiler makes
//! without it, as a click on the bar of another monitor, and their
//! reports, and focus follows mouse while a window is held.

use crate::action::Origin;
use crate::event::{DragStart, Edges, Grab};
use crate::id::wid;
use crate::tiler::TilingManager;
use crate::tiler::tests::fake_platform::{Call, FakePlatform};
use crate::tiler::tests::manager_support::{start, still};
use crate::{Action, Direction, WindowEvent};

use super::tests::{centered_on, moves_pointer};

/// The manager on windows 1 and 2 on the first monitor and window 3 on
/// the second, the pointer following focus, with window 1 held by `grab`.
fn holding(grab: Grab) -> (FakePlatform, TilingManager<FakePlatform>) {
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1)]);
    let mut manager = start(&platform, None, &still());
    manager.flush_pending();
    let start = DragStart {
        grab: Some(grab),
        ..DragStart::default()
    };
    let window = wid(1);
    manager.handle_event(&WindowEvent::MoveStarted { window, start });
    platform.take_calls();
    (platform, manager)
}

/// A click on the second monitor's bar moves focus there, which window 3
/// reports, then a key gives focus to window 2, and the user lets go of
/// window 1. Returns the calls before it is let go and those after.
fn bar_then_key(
    platform: &FakePlatform,
    manager: &mut TilingManager<FakePlatform>,
) -> [Vec<Call>; 2] {
    manager.handle_action_from(&Action::CycleLayout, Origin::Bar(1));
    manager.flush_pending();
    manager.handle_event(&WindowEvent::Focused { window: wid(3) });
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();
    let held = platform.take_calls();
    manager.handle_event(&WindowEvent::moved(wid(1)));
    manager.flush_pending();
    [held, platform.take_calls()]
}

#[test]
fn a_bar_moving_focus_mid_drag_leaves_the_pointer_held_through_its_report() {
    // Arrange -- window 1 held by its title bar.
    let (platform, mut manager) = holding(Grab::TitleBar);

    // Act
    let [held, dropped] = bar_then_key(&platform, &mut manager);

    // Assert -- the pointer goes to window 2 only once window 1 is let go.
    assert!(!moves_pointer(&held), "{held:?}");
    let at = platform.window(2).rect;
    assert!(dropped.contains(&centered_on(at)), "{dropped:?}");
}

#[test]
fn a_bar_moving_focus_mid_resize_leaves_the_pointer_held_through_its_report() {
    // Arrange -- window 1 held by its right edge.
    let right = Edges {
        right: true,
        ..Edges::default()
    };
    let (platform, mut manager) = holding(Grab::Edges(right));

    // Act
    let [held, dropped] = bar_then_key(&platform, &mut manager);

    // Assert
    assert!(!moves_pointer(&held), "{held:?}");
    let at = platform.window(2).rect;
    assert!(dropped.contains(&centered_on(at)), "{dropped:?}");
}

#[test]
fn focus_follows_mouse_waits_until_the_held_window_is_let_go() {
    // Arrange -- window 1 held by its title bar.
    let (platform, mut manager) = holding(Grab::TitleBar);

    // Act -- the pointer rests on window 2 while window 1 is held, then
    // again once it is let go.
    manager.handle_event(&WindowEvent::MouseHover { window: wid(2) });
    manager.flush_pending();
    let held = platform.take_calls();
    manager.handle_event(&WindowEvent::moved(wid(1)));
    manager.handle_event(&WindowEvent::MouseHover { window: wid(2) });
    manager.flush_pending();
    let after = platform.take_calls();

    // Assert
    assert!(!held.contains(&Call::Focus(wid(2))), "{held:?}");
    assert!(after.contains(&Call::Focus(wid(2))), "{after:?}");
}

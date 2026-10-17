//! What gives the pointer back before the end of a drag is reported: the
//! dragged window leaving, and another window taking focus other than by
//! the tiler's doing.

use crate::id::wid;
use crate::tiler::TilingManager;
use crate::tiler::tests::fake_platform::{Call, FakePlatform};
use crate::tiler::tests::manager_support::{start, still};
use crate::{Action, Direction, WindowEvent};

use super::tests::{centered_on, moves_pointer};

/// A way the dragged window leaves, as the platform has it and reports it.
type Leave = fn(&FakePlatform) -> WindowEvent;

/// The pointer following focus, the user drags window 1, which has
/// focus, beside window 2.
fn dragging() -> (FakePlatform, TilingManager<FakePlatform>) {
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &still());
    manager.flush_pending();
    manager.handle_event(&WindowEvent::move_started(wid(1)));
    (platform, manager)
}

#[test]
fn a_dragged_window_that_closes_hides_or_is_minimized_gives_the_pointer_back() {
    // Arrange -- each way the window leaves before the drag is reported
    // to end.
    let leaves: [(&str, Leave); 3] = [
        ("closes", |p| {
            p.close_window(1);
            WindowEvent::Destroyed { window: wid(1) }
        }),
        ("hides", |p| {
            p.edit(1, |w| w.visible = false);
            WindowEvent::Hidden { window: wid(1) }
        }),
        ("is minimized", |p| {
            p.edit(1, |w| w.minimized = true);
            WindowEvent::Minimized { window: wid(1) }
        }),
    ];

    for (how, leave) in leaves {
        let (platform, mut manager) = dragging();
        platform.take_calls();

        // Act
        manager.handle_event(&leave(&platform));
        manager.flush_pending();

        // Assert -- focus goes to window 2, now on the whole monitor, and
        // the pointer with it.
        let calls = platform.take_calls();
        assert!(calls.contains(&Call::Focus(wid(2))), "{how}: {calls:?}");
        let whole = platform.window(2).rect;
        assert!(calls.contains(&centered_on(whole)), "{how}: {calls:?}");
    }
}

#[test]
fn another_window_taking_focus_during_a_drag_gives_the_pointer_back() {
    // Arrange -- the user clicks window 2.
    let (platform, mut manager) = dragging();
    manager.handle_event(&WindowEvent::Focused { window: wid(2) });
    platform.take_calls();

    // Act -- a key gives focus back to window 1, the pointer following.
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();

    // Assert
    let calls = platform.take_calls();
    let left = platform.window(1).rect;
    assert!(calls.contains(&centered_on(left)), "{calls:?}");
}

#[test]
fn the_window_the_tiler_gave_focus_reporting_it_keeps_the_pointer_held() {
    // Arrange -- a key gives focus to window 2 meanwhile.
    let (platform, mut manager) = dragging();
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.flush_pending();
    platform.take_calls();

    // Act -- window 2 reports the focus it was given, then the user drops
    // window 1.
    manager.handle_event(&WindowEvent::Focused { window: wid(2) });
    let reported = platform.take_calls();
    manager.handle_event(&WindowEvent::moved(wid(1)));

    // Assert -- the pointer goes to window 2 only then.
    assert!(!moves_pointer(&reported), "{reported:?}");
    let right = platform.window(2).rect;
    let calls = platform.take_calls();
    assert!(calls.contains(&centered_on(right)), "{calls:?}");
}

#[test]
fn the_dragged_window_reporting_focus_keeps_the_pointer_held() {
    // Arrange -- window 1 has focus, and the user grabs window 2, whose
    // focus is reported after the start of its drag.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &still());
    manager.flush_pending();
    manager.handle_event(&WindowEvent::move_started(wid(2)));
    manager.handle_event(&WindowEvent::Focused { window: wid(2) });
    platform.take_calls();

    // Act -- a key gives focus to window 1.
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();

    // Assert -- the pointer stays with the user.
    let calls = platform.take_calls();
    assert!(calls.contains(&Call::Focus(wid(1))), "{calls:?}");
    assert!(!moves_pointer(&calls), "{calls:?}");
}

#[test]
fn a_late_report_of_a_focus_change_the_tiler_made_keeps_the_pointer_held() {
    // Arrange -- keys give focus to window 2, then back to window 1, while
    // the user drags window 1.
    let (platform, mut manager) = dragging();
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.flush_pending();
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();

    // Act -- the report of the first change comes late, then the drop.
    manager.handle_event(&WindowEvent::Focused { window: wid(2) });
    manager.handle_event(&WindowEvent::moved(wid(1)));

    // Assert -- the pointer goes to window 1 once it is dropped.
    let calls = platform.take_calls();
    let left = platform.window(1).rect;
    assert_eq!(calls.last(), Some(&centered_on(left)), "{calls:?}");
}

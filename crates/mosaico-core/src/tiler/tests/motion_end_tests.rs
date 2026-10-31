//! What ends a motion, effect by effect.

use crate::config::{RuleAction, WindowRule};
use crate::id::wid;
use crate::tiler::tests::manager_support::tiled;
use crate::{Action, WindowEvent};

use super::{animated, placements, settle, swap, two};

#[test]
fn a_moving_window_that_is_hidden_stops_where_it_is() {
    // Arrange -- both windows on their way, then a switch to an empty
    // workspace hides them.
    let (platform, mut manager) = two(&animated());
    let start = [1, 2].map(|id| platform.window(id).rect);
    swap(&mut manager);

    // Act
    manager.handle_action(&Action::GoToWorkspace(2));
    settle(&platform, &mut manager);

    // Assert
    assert!(placements(&platform.take_calls()).is_empty());
    assert_eq!([1, 2].map(|id| platform.window(id).rect), start);
    assert!(!manager.wants_frames());
}

#[test]
fn a_moving_window_that_is_minimized_hidden_or_grabbed_stops_while_the_other_goes_on() {
    for stop in [
        Stop::Minimize,
        Stop::Grab,
        Stop::Destroy,
        Stop::Minimized,
        Stop::Hidden,
    ] {
        // Arrange -- window 1 is focused, and on its way right.
        let (platform, mut manager) = two(&animated());
        let left = platform.window(1).rect;
        swap(&mut manager);

        // Act
        match stop {
            Stop::Minimize => manager.handle_action(&Action::MinimizeFocused),
            Stop::Grab => manager.handle_event(&WindowEvent::move_started(wid(1))),
            Stop::Destroy => {
                platform.close_window(1);
                manager.handle_event(&WindowEvent::Destroyed { window: wid(1) });
            }
            // The user minimizes it, or its application hides it.
            Stop::Minimized => {
                platform.edit(1, |w| w.minimized = true);
                manager.handle_event(&WindowEvent::Minimized { window: wid(1) });
            }
            Stop::Hidden => {
                platform.edit(1, |w| w.visible = false);
                manager.handle_event(&WindowEvent::Hidden { window: wid(1) });
            }
        }
        settle(&platform, &mut manager);

        // Assert -- window 2 still gets to its slot, or fills the monitor
        // once window 1 has left the layout.
        let placed = placements(&platform.take_calls());
        assert_eq!(placed.len(), 1, "{stop:?}: {placed:?}");
        assert_eq!(placed[0].0, 2, "{stop:?}");
        assert_eq!(
            placed[0].1 == left,
            matches!(stop, Stop::Minimize | Stop::Grab),
            "{stop:?}"
        );
    }
}

/// What stops window 1 on its way.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Stop {
    Minimize,
    Grab,
    Destroy,
    Minimized,
    Hidden,
}

#[test]
fn a_moving_window_asked_to_close_keeps_moving() {
    // Arrange -- the application may refuse to close.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    swap(&mut manager);

    // Act
    manager.handle_action(&Action::CloseFocused);
    settle(&platform, &mut manager);

    // Assert
    assert_eq!(placements(&platform.take_calls()), [(2, left), (1, right)]);
}

#[test]
fn a_moving_window_the_rules_let_go_of_ends_in_its_slot() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    let right = platform.window(2).rect;
    swap(&mut manager);
    platform.edit(1, |w| {
        w.facts.iter_mut().for_each(|f| f.title = "let go".into())
    });
    let rule = WindowRule {
        match_class: None,
        match_title: Some("let go".into()),
        match_path: None,
        action: RuleAction::Ignore,
    };

    // Act
    manager.reload_rules(vec![rule]);

    // Assert -- placed at once, as it leaves; window 2 moves on to fill
    // the monitor.
    assert_eq!(placements(&platform.take_calls()), [(1, right)]);
    assert!(manager.wants_frames());
}

#[test]
fn a_hide_or_minimize_report_the_desktop_does_not_bear_out_leaves_the_window_moving() {
    for report in [
        WindowEvent::Hidden { window: wid(1) },
        WindowEvent::Minimized { window: wid(1) },
    ] {
        // Arrange -- window 1 is on its way right, as its application
        // recreates its surface, or once it was restored again from a
        // minimize reported late.
        let (platform, mut manager) = two(&animated());
        let right = platform.window(2).rect;
        swap(&mut manager);

        // Act -- reported hidden or minimized while it stays shown.
        manager.handle_event(&report);
        settle(&platform, &mut manager);

        // Assert -- it is still tiled, and ends in its slot.
        assert_eq!(tiled(&manager, 0), [2, 1], "{report:?}");
        assert_eq!(platform.window(1).rect, right, "{report:?}");
    }
}

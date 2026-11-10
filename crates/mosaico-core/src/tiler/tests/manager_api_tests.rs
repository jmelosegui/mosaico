//! The bar API's state the manager keeps: built again only when
//! something it shows may have changed, and handed out only when it did.

use crate::WindowEvent;
use crate::api::{Daemon, Theme};
use crate::id::wid;
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::{start, still};
use crate::{Action, Direction};

#[test]
fn the_first_state_is_always_handed_out_and_an_unchanged_one_never() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let (daemon, theme) = (Daemon::default(), Theme::default());

    // Act
    let first = manager.bar_state(&daemon, &theme).is_some();
    let second = manager.bar_state(&daemon, &theme).is_some();

    // Assert
    assert!(first);
    assert!(!second);
}

#[test]
fn the_focused_window_comes_with_its_title_and_program() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());

    // Act
    let state = manager.bar_state(&Daemon::default(), &Theme::default());

    // Assert
    let window = state.and_then(|s| s.focus.window.clone());
    let window = window.map(|w| (w.id, w.title, w.program));
    assert_eq!(window, Some((wid(1), "app".into(), "app".into())));
}

#[test]
fn a_new_title_of_the_focused_window_is_handed_out() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let (daemon, theme) = (Daemon::default(), Theme::default());
    manager.bar_state(&daemon, &theme);
    let retitle = |w: &mut crate::tiler::tests::FakeWindow| {
        if let Some(facts) = w.facts.as_mut() {
            facts.title = "renamed".into();
        }
    };
    platform.edit(1, retitle);

    // Act
    manager.handle_event(&WindowEvent::TitleChanged { window: wid(1) });
    let state = manager.bar_state(&daemon, &theme).cloned();

    // Assert
    let title = state.and_then(|s| s.focus.window).map(|w| w.title);
    assert_eq!(title.as_deref(), Some("renamed"));
}

#[test]
fn a_window_that_only_moves_changes_nothing_a_bar_shows() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &still());
    let (daemon, theme) = (Daemon::default(), Theme::default());
    manager.bar_state(&daemon, &theme);

    // Act
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(2) });
    let state = manager.bar_state(&daemon, &theme).is_some();

    // Assert
    assert!(!state);
}

#[test]
fn a_change_of_what_the_daemon_says_is_handed_out() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let theme = Theme::default();
    manager.bar_state(&Daemon::default(), &theme);
    let paused = Daemon {
        paused: true,
        ..Daemon::default()
    };

    // Act
    let state = manager.bar_state(&paused, &theme).cloned();

    // Assert
    assert!(state.is_some_and(|s| s.daemon.paused));
}

#[test]
fn focus_moving_to_another_monitor_is_handed_out() {
    // Arrange
    let platform = FakePlatform::new(2, &[(1, 0), (2, 1)]);
    let mut manager = start(&platform, None, &still());
    let (daemon, theme) = (Daemon::default(), Theme::default());
    manager.bar_state(&daemon, &theme);

    // Act
    manager.handle_action(&Action::Focus(Direction::Right));
    let state = manager.bar_state(&daemon, &theme).cloned();

    // Assert
    let state = state.unwrap_or_default();
    let second = state.monitors.get(1).map(|m| m.id.clone());
    assert_eq!(state.focus.monitor, second);
    assert_eq!(state.focus.window.map(|w| w.id), Some(wid(2)));
}

//! The tiling manager on a fake platform: what it takes from the config
//! and the window rules.

use crate::config::{Config, CornerStyle, WindowRule};
use crate::{Direction, LayoutKind};

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::{start, tiled};
use super::*;

/// Whether the pointer was moved after a keyboard focus change with
/// `config`.
fn pointer_moved_on_focus(config: &Config) -> bool {
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, config);
    manager.flush_pending();
    platform.take_calls();
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.flush_pending();
    platform
        .calls()
        .iter()
        .any(|c| matches!(c, Call::MoveCursor(..)))
}

#[test]
fn the_pointer_follows_keyboard_focus_by_default() {
    // Arrange
    let config = Config::default();

    // Act
    let moved = pointer_moved_on_focus(&config);

    // Assert
    assert!(moved);
}

#[test]
fn the_pointer_stays_when_it_does_not_follow_focus() {
    // Arrange -- focus following the mouse is a different setting.
    let mut config = Config::default();
    config.mouse.follows_focus = false;
    config.mouse.focus_follows_mouse = true;

    // Act
    let moved = pointer_moved_on_focus(&config);

    // Assert
    assert!(!moved);
}

#[test]
fn borders_and_corners_are_drawn_the_way_the_config_says() {
    // Arrange
    let mut config = Config::default();
    config.borders.width = 7;
    config.borders.corner_style = CornerStyle::Round;
    let platform = FakePlatform::new(1, &[(1, 0)]);

    // Act
    start(&platform, None, &config);

    // Assert
    let calls = platform.calls();
    assert!(
        calls.contains(&Call::SetCorners(wid(1), CornerStyle::Round)),
        "{calls:?}"
    );
    let drawn = calls.iter().find_map(|c| match c {
        Call::ShowBorder(w, style) if *w == wid(1) => Some(style.clone()),
        _ => None,
    });
    let drawn = drawn.expect("window 1 has a border");
    assert_eq!(drawn.width, 7);
    assert_eq!(drawn.radius, CornerStyle::Round.border_radius());
}

#[test]
fn workspaces_start_with_the_layouts_the_config_says() {
    // Arrange
    let mut config = Config::default();
    config.layout.default = LayoutKind::VerticalStack;
    config.workspaces.layouts.insert(2, LayoutKind::ThreeColumn);
    let platform = FakePlatform::new(1, &[(1, 0)]);

    // Act
    let manager = start(&platform, None, &config);

    // Assert
    let workspaces = manager.tiler().monitors()[0].workspaces();
    assert_eq!(workspaces[0].layout_kind(), LayoutKind::VerticalStack);
    assert_eq!(workspaces[1].layout_kind(), LayoutKind::ThreeColumn);
}

/// Windows 1 and 2 on one monitor, window 2 of the class "Floating".
fn with_a_floating_window() -> FakePlatform {
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    platform.edit(2, |w| {
        if let Some(facts) = w.facts.as_mut() {
            facts.class = "Floating".into();
        }
    });
    platform
}

/// A rule that leaves the windows of the class "Floating" alone.
fn floating() -> WindowRule {
    WindowRule {
        match_class: Some("Floating".into()),
        match_title: None,
        match_path: None,
        manage: false,
    }
}

#[test]
fn the_rules_it_starts_with_leave_windows_alone() {
    // Arrange
    let platform = with_a_floating_window();

    // Act
    let manager = TilingManager::new(platform.clone(), None, &Config::default(), vec![floating()])
        .expect("the manager starts");

    // Assert
    assert_eq!(tiled(&manager, 0), vec![1]);
}

#[test]
fn reloaded_rules_let_go_of_the_windows_they_leave_alone() {
    // Arrange -- no rules at first, so both windows are tiled.
    let platform = with_a_floating_window();
    let mut manager = start(&platform, None, &Config::default());
    let before = tiled(&manager, 0);

    // Act
    manager.reload_rules(vec![floating()]);

    // Assert
    assert_eq!(before, vec![1, 2]);
    assert_eq!(tiled(&manager, 0), vec![1]);
}

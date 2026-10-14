//! The tiling manager on a fake platform: what it takes from the config.

use crate::config::{Config, CornerStyle};
use crate::{Direction, LayoutKind};

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::start;
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

//! Borders held behind a window's move, when a step that does not move
//! the window decides them again before the move lands (#133).

use std::collections::HashMap;

use crate::Direction;
use crate::config::{Color, Config};

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::start;
use super::*;

const FOCUSED: Color = Color {
    r: 0x11,
    g: 0x11,
    b: 0x11,
};
const UNFOCUSED: Color = Color {
    r: 0x22,
    g: 0x22,
    b: 0x22,
};

/// Two windows side by side, the left one focused, with focused and
/// unfocused borders in different colors.
fn side_by_side() -> (FakePlatform, TilingManager<FakePlatform>) {
    let mut config = Config::default();
    config.borders.colors.focused = "#111111".into();
    config.borders.colors.unfocused = "#222222".into();
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let manager = start(&platform, None, &config);
    (platform, manager)
}

/// The style each window's border was last drawn with.
fn last_drawn(platform: &FakePlatform) -> HashMap<u64, BorderStyle> {
    let mut last = HashMap::new();
    for call in platform.calls() {
        if let Call::ShowBorder(window, style) = call {
            last.insert(window.raw(), style);
        }
    }
    last
}

/// Asserts that windows 1 and 2 swapped their slots, `left` and
/// `right`, as the held move says.
fn assert_swapped(platform: &FakePlatform, left: Rect, right: Rect) {
    assert_eq!(
        platform.window(1).rect,
        right,
        "window 1 did not move right"
    );
    assert_eq!(platform.window(2).rect, left, "window 2 did not move left");
}

/// Asserts that `window`'s border was last drawn around the slot the
/// window is in, in `color`.
fn assert_border_around(platform: &FakePlatform, window: u64, color: Color) {
    let drawn = last_drawn(platform);
    let style = &drawn[&window];
    assert_eq!(
        style.rect,
        platform.window(window).rect,
        "border of {window} drawn away from it"
    );
    assert_eq!(style.color, color, "border of {window} in the wrong color");
}

#[test]
fn a_focus_change_before_a_held_move_lands_draws_the_borders_around_the_new_slots() {
    // #133
    // Arrange
    let (platform, mut manager) = side_by_side();
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);

    // Act -- the move is held until input goes quiet, and focus moves
    // to the window now on the left before that.
    manager.handle_action(&Action::Move(Direction::Right));
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();

    // Assert
    assert_swapped(&platform, left, right);
    assert_border_around(&platform, 1, UNFOCUSED);
    assert_border_around(&platform, 2, FOCUSED);
}

#[test]
fn a_border_refresh_before_a_held_move_lands_draws_the_borders_around_the_new_slots() {
    // #133
    // Arrange
    let (platform, mut manager) = side_by_side();
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);

    // Act
    manager.handle_action(&Action::Move(Direction::Right));
    manager.refresh_border();
    manager.flush_pending();

    // Assert -- window 1 keeps focus.
    assert_swapped(&platform, left, right);
    assert_border_around(&platform, 1, FOCUSED);
    assert_border_around(&platform, 2, UNFOCUSED);
}

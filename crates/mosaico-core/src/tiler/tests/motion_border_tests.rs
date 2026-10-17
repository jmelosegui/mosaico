//! The borders of moving windows: drawn once each window is in place.

use crate::config::Color;
use crate::id::wid;
use crate::tiler::tests::displays;
use crate::tiler::tests::fake_platform::Call;
use crate::{Action, Direction, Rect, WindowEvent};

use super::{animated, settle, swap, two};

/// The border calls among `calls`: each window, with the rect its border
/// was drawn around, or `None` where it was hidden.
fn borders(calls: &[Call]) -> Vec<(u64, Option<Rect>)> {
    let border = |call: &Call| match call {
        Call::ShowBorder(window, style) => Some((window.raw(), Some(style.rect))),
        Call::HideBorder(window) => Some((window.raw(), None)),
        _ => None,
    };
    calls.iter().filter_map(border).collect()
}

#[test]
fn a_moving_window_has_its_border_drawn_once_it_is_in_place() {
    // Arrange -- two slots of the same size, so the borders move as they
    // are, without being drawn from scratch.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    assert_eq!((left.width, left.height), (right.width, right.height));
    platform.take_renders();

    // Act
    swap(&mut manager);
    let on_the_way = borders(&platform.take_calls());
    settle(&platform, &mut manager);

    // Assert -- each as its motion ends, with its window there already.
    assert!(on_the_way.is_empty(), "{on_the_way:?}");
    let in_place = borders(&platform.take_calls());
    assert_eq!(in_place, [(2, Some(left)), (1, Some(right))]);
    assert!(platform.take_renders().is_empty());
}

#[test]
fn a_window_that_changes_size_on_its_way_has_its_border_hidden_until_then() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    platform.take_renders();

    // Act -- window 1 grows to fill the monitor.
    manager.handle_action(&Action::ToggleMonocle);
    let on_the_way = borders(&platform.take_calls());
    settle(&platform, &mut manager);

    // Assert -- window 2 is hidden behind it in monocle anyway.
    assert_eq!(on_the_way, [(1, None), (2, None)]);
    let monocle = platform.window(1).rect;
    assert_eq!(borders(&platform.take_calls()), [(1, Some(monocle))]);
    assert_eq!(platform.take_renders(), [1]);
}

#[test]
fn a_border_to_be_hidden_while_its_window_moves_is_hidden_at_once() {
    // Arrange -- only the focused window has a border. Window 1 is on its
    // way right, window 2 left.
    let mut config = animated();
    config.borders.colors.unfocused = "none".into();
    let (platform, mut manager) = two(&config);
    let left = platform.window(1).rect;
    swap(&mut manager);
    platform.take_calls();

    // Act -- focus goes to window 2.
    manager.handle_action(&Action::Focus(Direction::Left));
    let at_once = borders(&platform.take_calls());
    settle(&platform, &mut manager);

    // Assert
    assert_eq!(at_once, [(1, None)]);
    assert_eq!(borders(&platform.take_calls()), [(2, Some(left))]);
}

#[test]
fn a_window_stopped_on_its_way_has_no_border_drawn_for_it() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    let left = platform.window(1).rect;
    swap(&mut manager);

    // Act -- the user grabs window 1.
    manager.handle_event(&WindowEvent::move_started(wid(1)));
    settle(&platform, &mut manager);

    // Assert
    assert_eq!(borders(&platform.take_calls()), [(2, Some(left))]);
}

#[test]
fn a_window_a_display_change_stopped_has_its_border_drawn_once_it_is_placed() {
    // Arrange -- two slots of the same size, the focused border red and
    // the others green. The windows swap as the change is noted, and focus
    // moves to window 2 while it settles.
    let mut config = animated();
    config.borders.colors.focused = "#ff0000".into();
    config.borders.colors.unfocused = "#00ff00".into();
    let (platform, mut manager) = two(&config);
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    swap(&mut manager);
    manager.display_changing();
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();
    platform.take_calls();

    // Act -- the change settles back where it started.
    manager.handle_display_change(displays(1), 0, &[]);

    // Assert -- each border is drawn once, at the window's target, as the
    // focus change left it.
    let calls = platform.take_calls();
    assert_eq!(borders(&calls), [(2, Some(left)), (1, Some(right))]);
    let color = |id| {
        let drawn = |call: &Call| match call {
            Call::ShowBorder(window, style) if *window == wid(id) => Some(style.color),
            _ => None,
        };
        calls.iter().find_map(drawn)
    };
    assert_eq!(color(2), Color::from_hex("#ff0000"));
    assert_eq!(color(1), Color::from_hex("#00ff00"));
}

#[test]
fn a_window_placed_while_a_display_change_settles_moves_next_with_a_fresh_border() {
    // Arrange -- window 1 grows to fill the monitor, its border hidden on
    // the way, when a display change stops it. Monocle is turned off while
    // the change settles, which places window 1 at once, and the change
    // settles back where it started.
    let (platform, mut manager) = two(&animated());
    manager.handle_action(&Action::ToggleMonocle);
    manager.display_changing();
    manager.handle_action(&Action::ToggleMonocle);
    manager.handle_display_change(displays(1), 0, &[]);
    platform.take_calls();

    // Act -- it grows again.
    manager.handle_action(&Action::ToggleMonocle);

    // Assert -- its border is hidden on the way again.
    assert_eq!(borders(&platform.take_calls()), [(1, None), (2, None)]);
}

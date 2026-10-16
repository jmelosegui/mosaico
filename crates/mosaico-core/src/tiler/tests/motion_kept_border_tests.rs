//! The border a motion kept for a window whose final placement is on its
//! way, or one the tiler decided meanwhile: drawn once the window reports
//! being there, around where it was placed, unless the tiler hid or
//! minimized the window, or set it moving again.

use crate::config::Color;
use crate::id::wid;
use crate::tiler::BorderStyle;
use crate::tiler::tests::fake_desktop::Queued;
use crate::tiler::tests::fake_platform::Call;
use crate::{Action, Direction, WindowEvent};

use super::{FakePlatform, Manager, REFRESH, animated, frame_by_frame, swap, two};

/// Window 1, focused, stops responding once it got to the first frame of a
/// swap, which ends the swap with its placement queued behind the frame,
/// and its border kept to draw once it lands. Focused borders are red, and
/// the others green.
fn hung_after_first_frame() -> (FakePlatform, Manager) {
    hung_after_first_frame_with("#00ff00")
}

/// As [`hung_after_first_frame`], with the others `unfocused`.
fn hung_after_first_frame_with(unfocused: &str) -> (FakePlatform, Manager) {
    let mut config = animated();
    config.borders.colors.focused = "#ff0000".into();
    config.borders.colors.unfocused = unfocused.into();
    let (platform, mut manager) = two(&config);
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();
    platform.edit(1, |w| w.hung = true);
    frame_by_frame(&platform, &mut manager);
    (platform, manager)
}

/// The border drawn last for window 1 among `calls`, if one was.
fn drawn(calls: &[Call]) -> Option<BorderStyle> {
    let one = |call: &Call| match call {
        Call::ShowBorder(w, style) if *w == wid(1) => Some(style.clone()),
        _ => None,
    };
    calls.iter().rev().find_map(one)
}

/// Window 1's application responds again, gets to what is queued for it,
/// and reports where that leaves the window. Gives the border drawn for
/// window 1 then, if one was.
fn lands(platform: &FakePlatform, manager: &mut Manager) -> Option<BorderStyle> {
    platform.take_calls();
    platform.edit(1, |w| w.hung = false);
    platform.catch_up(1);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });
    drawn(&platform.take_calls())
}

#[test]
fn a_border_decided_while_the_placement_is_on_its_way_is_drawn_once_the_window_lands() {
    // Arrange -- focus moves to window 2, so window 1's border turns green.
    let (platform, mut manager) = hung_after_first_frame();
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();
    let decided = platform.take_calls();

    // Act
    let landed = lands(&platform, &mut manager);

    // Assert -- not where the desktop still has the window, but where it
    // lands, in the color decided.
    assert_eq!(drawn(&decided), None, "{decided:?}");
    let style = landed.expect("a border drawn as window 1 lands");
    assert_eq!(style.rect, platform.window(1).rect);
    assert_eq!(Some(style.color), Color::from_hex("#00ff00"));
}

#[test]
fn a_border_hidden_while_the_placement_is_on_its_way_is_hidden_at_once_and_stays_so() {
    // Arrange -- only the focused window has a border, and focus moves to
    // window 2.
    let (platform, mut manager) = hung_after_first_frame_with("none");
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.flush_pending();
    let decided = platform.take_calls();

    // Act
    let landed = lands(&platform, &mut manager);

    // Assert
    assert!(decided.contains(&Call::HideBorder(wid(1))), "{decided:?}");
    assert_eq!(landed, None);
}

#[test]
fn a_window_minimized_while_its_placement_is_on_its_way_gets_no_border() {
    // Arrange -- the user minimizes it with a hotkey, which the tiler asks
    // the platform for.
    let (platform, mut manager) = hung_after_first_frame();
    manager.handle_action(&Action::MinimizeFocused);

    // Act
    let drawn = lands(&platform, &mut manager);

    // Assert
    assert_eq!(drawn, None);
}

#[test]
fn a_window_that_moves_on_before_its_placement_lands_gets_no_border_as_it_lands() {
    // Arrange -- the swap is undone, which sets window 1 moving back.
    let (platform, mut manager) = hung_after_first_frame();
    let queued = platform.window(1).queued;
    let place = queued
        .iter()
        .map(|&(q, _)| q)
        .find(|q| matches!(q, Queued::Place(_)));
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();

    // Act -- its application gets to the placement of the first motion
    // only, and reports where that leaves the window.
    platform.take_calls();
    platform.edit(1, |w| w.hung = false);
    platform.catch_up_through(1, place.expect("a placement on its way"));
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });

    // Assert -- no border is drawn while it moves.
    let calls = platform.take_calls();
    assert_eq!(drawn(&calls), None, "{calls:?}");
}

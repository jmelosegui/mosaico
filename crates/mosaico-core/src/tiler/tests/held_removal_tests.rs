//! Moves held for a window that leaves the layout before input goes
//! quiet (#135).

use crate::Direction;
use crate::config::{Config, RuleAction, WindowRule};

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::{start, still};
use super::*;

#[path = "held_removal_tiled_tests.rs"]
mod tiled_tests;

/// Windows on `monitors` monitors, the first one focused, with no border
/// around the windows without focus: the others have never had one. The
/// pointer does not follow focus, and transitions are off.
fn without_unfocused_borders(
    monitors: usize,
    windows: &[(u64, usize)],
) -> (FakePlatform, TilingManager<FakePlatform>) {
    without_unfocused_borders_in(still(), monitors, windows)
}

/// The same, managed with `config` otherwise.
fn without_unfocused_borders_in(
    mut config: Config,
    monitors: usize,
    windows: &[(u64, usize)],
) -> (FakePlatform, TilingManager<FakePlatform>) {
    config.borders.colors.unfocused = "none".into();
    config.mouse.follows_focus = false;
    let platform = FakePlatform::new(monitors, windows);
    let manager = start(&platform, None, &config);
    (platform, manager)
}

/// Swaps windows 1 and 2, whose moves wait for input to go quiet.
fn hold_a_swap(manager: &mut TilingManager<FakePlatform>) {
    manager.handle_action(&Action::Move(Direction::Right));
    assert!(manager.held_since().is_some(), "the moves were not held");
}

/// Everything the platform is asked to do when the held effects are
/// applied.
fn flush(platform: &FakePlatform, manager: &mut TilingManager<FakePlatform>) -> Vec<Call> {
    platform.take_calls();
    manager.flush_pending();
    platform.take_calls()
}

#[test]
fn a_window_minimized_while_its_move_is_held_is_left_alone() {
    // Arrange -- window 2 has never had a border to drop (#135).
    let (platform, mut manager) = without_unfocused_borders(1, &[(1, 0), (2, 0)]);
    hold_a_swap(&mut manager);

    // Act -- the user minimizes it before input goes quiet.
    platform.edit(2, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(2) });
    let calls = flush(&platform, &mut manager);

    // Assert -- only the focus change held with the moves is left, since
    // window 1 already took the whole monitor.
    assert_eq!(calls, [Call::Focus(wid(1))]);
    assert!(platform.window(2).minimized);
}

#[test]
fn a_window_hidden_while_its_move_is_held_is_left_alone() {
    // Arrange
    let (platform, mut manager) = without_unfocused_borders(1, &[(1, 0), (2, 0)]);
    hold_a_swap(&mut manager);

    // Act -- closed to the tray, say.
    platform.edit(2, |w| w.visible = false);
    manager.handle_event(&WindowEvent::Hidden { window: wid(2) });
    let calls = flush(&platform, &mut manager);

    // Assert
    assert_eq!(calls, [Call::Focus(wid(1))]);
}

#[test]
fn a_window_the_rules_let_go_while_its_move_is_held_is_left_alone() {
    // Arrange
    let (platform, mut manager) = without_unfocused_borders(1, &[(1, 0), (2, 0)]);
    hold_a_swap(&mut manager);
    platform.edit(2, |w| {
        if let Some(facts) = w.facts.as_mut() {
            facts.class = "Excluded".into();
        }
    });

    // Act -- new rules stop managing it before input goes quiet.
    manager.reload_rules(vec![WindowRule {
        match_class: Some("Excluded".into()),
        match_title: None,
        match_path: None,
        action: RuleAction::Ignore,
    }]);
    let calls = flush(&platform, &mut manager);

    // Assert
    assert_eq!(calls, [Call::Focus(wid(1))]);
}

#[test]
fn a_window_a_retile_prunes_while_its_move_is_held_is_left_alone() {
    // Arrange
    let (platform, mut manager) = without_unfocused_borders(1, &[(1, 0), (2, 0)]);
    hold_a_swap(&mut manager);

    // Act -- it was minimized without a report, and a retile before input
    // goes quiet drops it from the layout.
    platform.edit(2, |w| w.minimized = true);
    manager.handle_action(&Action::Retile);
    let calls = flush(&platform, &mut manager);

    // Assert
    assert_eq!(calls, [Call::Focus(wid(1))]);
}

/// One window on each of two monitors, the left one focused, with the
/// pointer following focus.
fn one_window_per_monitor() -> (FakePlatform, TilingManager<FakePlatform>) {
    let platform = FakePlatform::new(2, &[(1, 0), (2, 1)]);
    let manager = start(&platform, None, &Config::default());
    (platform, manager)
}

/// Moves focus to window 2, a change that waits for input to go quiet.
fn hold_focus_on_the_right(manager: &mut TilingManager<FakePlatform>) {
    manager.handle_action(&Action::Focus(Direction::Right));
    assert!(
        manager.held_since().is_some(),
        "the focus change was not held"
    );
}

#[test]
fn a_window_minimized_while_focus_on_it_is_held_is_not_focused() {
    // Arrange
    let (platform, mut manager) = one_window_per_monitor();
    hold_focus_on_the_right(&mut manager);

    // Act
    platform.edit(2, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(2) });
    let calls = flush(&platform, &mut manager);

    // Assert -- no focus change, and the pointer stays where it is.
    assert_eq!(calls, Vec::<Call>::new());
}

#[test]
fn a_window_a_retile_prunes_while_focus_on_it_is_held_is_not_focused() {
    // Arrange
    let (platform, mut manager) = one_window_per_monitor();
    hold_focus_on_the_right(&mut manager);

    // Act -- minimized without a report, then dropped by a retile.
    platform.edit(2, |w| w.minimized = true);
    manager.handle_action(&Action::Retile);
    let calls = flush(&platform, &mut manager);

    // Assert
    assert_eq!(calls, Vec::<Call>::new());
}

#[test]
fn a_window_destroyed_while_focus_on_it_is_held_is_not_focused() {
    // Arrange
    let (platform, mut manager) = one_window_per_monitor();
    hold_focus_on_the_right(&mut manager);

    // Act
    platform.close_window(2);
    manager.handle_event(&WindowEvent::Destroyed { window: wid(2) });
    let calls = flush(&platform, &mut manager);

    // Assert
    assert_eq!(calls, Vec::<Call>::new());
}

#[test]
fn a_focus_report_after_a_focus_change_was_dropped_is_followed() {
    // Arrange -- window 2 is minimized while focus on it is held.
    let (platform, mut manager) = one_window_per_monitor();
    hold_focus_on_the_right(&mut manager);
    platform.edit(2, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(2) });
    flush(&platform, &mut manager);

    // Act -- the user clicks window 1.
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });

    // Assert -- not taken for the system moving focus while a focus
    // change of ours waits.
    assert_eq!(manager.tiler().focused_window(), Some(wid(1)));
}

#[test]
fn a_window_destroyed_while_its_move_is_held_is_left_alone() {
    // Arrange
    let (platform, mut manager) = without_unfocused_borders(1, &[(1, 0), (2, 0)]);
    hold_a_swap(&mut manager);

    // Act
    platform.close_window(2);
    manager.handle_event(&WindowEvent::Destroyed { window: wid(2) });
    let calls = flush(&platform, &mut manager);

    // Assert
    assert_eq!(calls, [Call::Focus(wid(1))]);
}

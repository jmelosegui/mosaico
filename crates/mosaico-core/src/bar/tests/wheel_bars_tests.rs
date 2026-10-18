//! When the wheel's turns are due, on each bar, and on bars that turn it
//! off or are gone.

use std::time::Instant;

use crate::Action;
use crate::action::Origin;
use crate::bar::WHEEL_QUIET;
use crate::bar::fakes::FakeRenderer;
use crate::config::BarConfig;

use super::super::tests::{MOCHA, bar, left, right};
use super::support::{bar_on, drawn_with, showing, turn};

#[test]
fn the_wheel_is_due_once_quiet_after_the_last_turn() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let last = turn(&mut manager, first, &[1.0, 1.0], Instant::now());

    // Assert
    assert_eq!(manager.wheel_due(), Some(last + WHEEL_QUIET));
    manager.take_wheel(last + WHEEL_QUIET);
    assert_eq!(manager.wheel_due(), None);
}

#[test]
fn each_bar_adds_up_its_own_turns() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let (first, second) = (bar_on(&renderer, 0), bar_on(&renderer, 1));
    let now = Instant::now();

    // Act
    turn(&mut manager, first, &[1.0], now);
    let last = turn(&mut manager, second, &[1.0, 1.0], now);
    let switches = manager.take_wheel(last + WHEEL_QUIET);

    // Assert
    assert_eq!(
        switches,
        [
            (Action::GoToWorkspace(2), Origin::Bar(0)),
            (Action::GoToWorkspace(3), Origin::Bar(1)),
        ]
    );
}

#[test]
fn with_scroll_none_the_wheel_does_nothing() {
    // Arrange
    let renderer = FakeRenderer::default();
    let config = bar("[mouse]\nscroll = \"none\"");
    let mut manager = drawn_with(config, &showing(&[0, 0]), &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let last = turn(&mut manager, first, &[1.0], Instant::now());

    // Assert
    assert_eq!(manager.wheel_due(), None);
    assert!(manager.take_wheel(last + WHEEL_QUIET).is_empty());
}

#[test]
fn turns_on_bars_made_again_since_are_dropped() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let first = bar_on(&renderer, 0);
    let last = turn(&mut manager, first, &[1.0], Instant::now());

    // Act
    manager.rebuild_for_monitors(vec![left(), right()], MOCHA);
    let late = turn(&mut manager, first, &[1.0], last);

    // Assert -- neither the turn before nor the one after counts.
    assert_eq!(manager.wheel_due(), None);
    assert!(manager.take_wheel(late + WHEEL_QUIET).is_empty());
}

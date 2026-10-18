//! The wheel over a bar, as the manager adds its turns up and turns
//! them into a workspace switch.

use std::time::{Duration, Instant};

use crate::Action;
use crate::action::Origin;
use crate::bar::WHEEL_QUIET;
use crate::bar::fakes::FakeRenderer;
use crate::config::BarConfig;

use super::support::{bar_on, drawn_with, showing, turn};

#[test]
fn a_burst_of_notches_switches_once_when_the_wheel_goes_quiet() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let last = turn(&mut manager, first, &[1.0, 1.0, 1.0], Instant::now());
    let early = manager.take_wheel(last + WHEEL_QUIET - Duration::from_millis(1));
    let quiet = manager.take_wheel(last + WHEEL_QUIET);

    // Assert
    assert!(early.is_empty());
    assert_eq!(quiet, [(Action::GoToWorkspace(4), Origin::Bar(0))]);
}

#[test]
fn the_wheel_steps_from_the_workspace_the_bar_shows() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 3]), &renderer);
    let second = bar_on(&renderer, 1);

    // Act
    let last = turn(&mut manager, second, &[-1.0], Instant::now());
    let switches = manager.take_wheel(last + WHEEL_QUIET);

    // Assert -- up goes to the workspace before, on monitor 1.
    assert_eq!(switches, [(Action::GoToWorkspace(3), Origin::Bar(1))]);
}

#[test]
fn turning_past_the_last_workspace_without_wrap_switches_nothing() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[7, 7]), &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let last = turn(&mut manager, first, &[1.0], Instant::now());
    let switches = manager.take_wheel(last + WHEEL_QUIET);

    // Assert
    assert!(switches.is_empty());
}

#[test]
fn with_wrap_turning_past_the_last_workspace_goes_to_the_first() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut states = showing(&[7, 7]);
    states[0].wrap = true;
    let mut manager = drawn_with(BarConfig::default(), &states, &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let last = turn(&mut manager, first, &[1.0], Instant::now());
    let switches = manager.take_wheel(last + WHEEL_QUIET);

    // Assert
    assert_eq!(switches, [(Action::GoToWorkspace(1), Origin::Bar(0))]);
}

#[test]
fn a_touchpads_fractions_add_up_to_a_switch() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[2, 0]), &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let last = turn(&mut manager, first, &[0.3, 0.4, 0.5, 0.2], Instant::now());
    let switches = manager.take_wheel(last + WHEEL_QUIET);

    // Assert -- 1.4 notches is one step.
    assert_eq!(switches, [(Action::GoToWorkspace(4), Origin::Bar(0))]);
}

#[test]
fn turns_that_cancel_out_switch_nothing() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[2, 0]), &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let last = turn(&mut manager, first, &[1.0, -1.0], Instant::now());
    let switches = manager.take_wheel(last + WHEEL_QUIET);

    // Assert
    assert!(switches.is_empty());
}

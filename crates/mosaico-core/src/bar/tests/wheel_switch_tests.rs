//! Turns over a bar whose monitor is switched some other way while they
//! wait: the burst they began stays dropped, fraction and all.

use std::time::{Duration, Instant};

use crate::bar::WHEEL_QUIET;
use crate::bar::fakes::FakeRenderer;
use crate::config::BarConfig;

use super::support::{bar_on, drawn_with, showing, turn};

#[test]
fn a_turn_after_a_switch_made_meanwhile_does_not_save_its_burst() {
    // Arrange -- a notch over monitor 0's bar, then a key takes monitor 0
    // to 5, then another notch, all before the wheel goes quiet.
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let first = bar_on(&renderer, 0);
    let start = Instant::now();
    turn(&mut manager, first, &[1.0], start);
    manager.update(&showing(&[4, 0]));
    let last = turn(
        &mut manager,
        first,
        &[1.0],
        start + Duration::from_millis(50),
    );

    // Act
    let switches = manager.take_wheel(last + WHEEL_QUIET);

    // Assert -- the burst began on 1, which the bar no longer shows.
    assert!(switches.is_empty());
}

#[test]
fn a_burst_dropped_for_a_switch_made_meanwhile_leaves_no_fraction() {
    // Arrange -- 0.6 of a notch, then a key switch before the wheel goes
    // quiet, which drops it.
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let first = bar_on(&renderer, 0);
    let last = turn(&mut manager, first, &[0.6], Instant::now());
    manager.update(&showing(&[4, 0]));
    manager.take_wheel(last + WHEEL_QUIET);

    // Act -- half a notch more, soon after.
    let later = turn(&mut manager, first, &[0.5], last + WHEEL_QUIET * 2);
    let switches = manager.take_wheel(later + WHEEL_QUIET);

    // Assert -- half a notch alone makes no step.
    assert!(switches.is_empty());
}

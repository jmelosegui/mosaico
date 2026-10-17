//! The fraction of a notch left over once turns are applied, and how
//! long it is kept.

use std::time::{Duration, Instant};

use super::tests::{FAST, turned};
use super::*;

/// Applies each of `turns` on its own, `apart` from one another from
/// `start`, once the wheel went quiet after it, and returns the steps.
fn one_by_one(wheel: &mut WheelSteps, turns: &[f32], start: Instant, apart: Duration) -> Vec<i32> {
    let mut steps = Vec::new();
    for (i, &turn) in turns.iter().enumerate() {
        let at = start + apart * i as u32;
        wheel.turn(turn, at);
        steps.extend(wheel.take(at + WHEEL_QUIET));
    }
    steps
}

#[test]
fn a_fraction_left_over_adds_up_with_the_next_turns() {
    // Arrange
    let (mut wheel, last) = turned(&[0.3, 0.3], Instant::now(), FAST);

    // Act
    let steps = wheel.take(last + WHEEL_QUIET);
    wheel.turn(0.5, last + WHEEL_QUIET * 2);
    let next = wheel.take(last + WHEEL_QUIET * 3);

    // Assert -- 0.6, then 0.5 more: 1.1 notches.
    assert_eq!((steps, next), (Some(0), Some(1)));
}

#[test]
fn a_slow_high_resolution_wheel_switches_once_its_eighths_make_a_notch() {
    // Arrange -- an eighth of a notch at a time, each one after the wheel
    // went quiet.
    let mut wheel = WheelSteps::default();

    // Act
    let steps = one_by_one(&mut wheel, &[0.125; 8], Instant::now(), WHEEL_QUIET * 2);

    // Assert
    assert_eq!(steps, [0, 0, 0, 0, 0, 0, 0, 1]);
}

#[test]
fn a_fraction_left_over_is_dropped_when_the_wheel_turns_the_other_way() {
    // Arrange
    let mut wheel = WheelSteps::default();

    // Act -- 0.6 down, then 1.0 up, half a notch at a time.
    let steps = one_by_one(
        &mut wheel,
        &[0.6, -0.5, -0.5],
        Instant::now(),
        WHEEL_QUIET * 2,
    );

    // Assert -- the 0.6 does not take a tenth off the way up.
    assert_eq!(steps, [0, 0, -1]);
}

#[test]
fn a_fraction_left_over_is_dropped_after_a_second_without_turns() {
    // Arrange
    let mut wheel = WheelSteps::default();

    // Act
    let steps = one_by_one(
        &mut wheel,
        &[0.6, 0.5],
        Instant::now(),
        Duration::from_secs(1),
    );

    // Assert
    assert_eq!(steps, [0, 0]);
}

#[test]
fn clearing_forgets_the_fraction_left_over() {
    // Arrange
    let start = Instant::now();
    let mut wheel = WheelSteps::default();
    one_by_one(&mut wheel, &[0.6], start, FAST);

    // Act
    wheel.clear();
    let steps = one_by_one(&mut wheel, &[0.5], start + WHEEL_QUIET * 2, FAST);

    // Assert
    assert_eq!(steps, [0]);
}

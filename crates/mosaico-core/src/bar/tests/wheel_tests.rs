use std::time::{Duration, Instant};

use super::*;

/// Turns of `notches` each, `apart` from one another, from `start`.
pub(super) fn turned(notches: &[f32], start: Instant, apart: Duration) -> (WheelSteps, Instant) {
    let mut wheel = WheelSteps::default();
    let mut at = start;
    for (i, &turn) in notches.iter().enumerate() {
        at = start + apart * i as u32;
        wheel.turn(turn, at);
    }
    (wheel, at)
}

pub(super) const FAST: Duration = Duration::from_millis(15);

#[test]
fn notches_add_up() {
    // Arrange
    let (mut wheel, last) = turned(&[1.0, 1.0, 1.0], Instant::now(), FAST);

    // Act
    let steps = wheel.take(last + WHEEL_QUIET);

    // Assert
    assert_eq!(steps, Some(3));
}

#[test]
fn the_steps_wait_until_the_wheel_is_quiet() {
    // Arrange
    let (mut wheel, last) = turned(&[1.0, 1.0], Instant::now(), FAST);

    // Act
    let early = wheel.take(last + WHEEL_QUIET - Duration::from_millis(1));
    let due = wheel.take(last + WHEEL_QUIET);

    // Assert -- the early look takes nothing away.
    assert_eq!((early, due), (None, Some(2)));
}

#[test]
fn each_turn_pushes_the_quiet_time_back() {
    // Arrange
    let start = Instant::now();
    let (wheel, last) = turned(&[1.0, 1.0], start, Duration::from_millis(100));

    // Act
    let due = wheel.due();

    // Assert
    assert_eq!(due, Some(last + WHEEL_QUIET));
    assert_eq!(last, start + Duration::from_millis(100));
}

#[test]
fn fractions_of_a_notch_add_up_to_one() {
    // Arrange -- quarters, and nine tenths and a tenth, which add up to a
    // little less than one.
    let (mut quarters, a) = turned(&[0.25; 4], Instant::now(), FAST);
    let (mut tenths, b) = turned(&[0.9, 0.1], Instant::now(), FAST);

    // Act
    let steps = (quarters.take(a + WHEEL_QUIET), tenths.take(b + WHEEL_QUIET));

    // Assert
    assert_eq!(steps, (Some(1), Some(1)));
}

#[test]
fn a_change_of_direction_takes_off_what_went_the_other_way() {
    // Arrange
    let (mut down, a) = turned(&[1.0, 1.0, -1.0], Instant::now(), FAST);
    let (mut up, b) = turned(&[-1.0, -1.0, 0.5], Instant::now(), FAST);

    // Act
    let steps = (down.take(a + WHEEL_QUIET), up.take(b + WHEEL_QUIET));

    // Assert -- -1.5 is one whole step up.
    assert_eq!(steps, (Some(1), Some(-1)));
}

#[test]
fn a_burst_after_the_wheel_went_quiet_counts_on_its_own() {
    // Arrange
    let (mut wheel, last) = turned(&[1.0, 1.0], Instant::now(), FAST);
    wheel.take(last + WHEEL_QUIET);

    // Act
    let later = last + WHEEL_QUIET * 2;
    wheel.turn(-1.0, later);
    let steps = wheel.take(later + WHEEL_QUIET);

    // Assert
    assert_eq!(steps, Some(-1));
}

#[test]
fn a_turn_that_is_not_a_number_is_left_out() {
    // Arrange
    let (mut wheel, last) = turned(&[1.0, f32::NAN, f32::INFINITY], Instant::now(), FAST);

    // Act -- the last turn that counted was the first.
    let steps = wheel.take(last + WHEEL_QUIET);

    // Assert
    assert_eq!(steps, Some(1));
}

#[test]
fn without_turns_nothing_is_due() {
    // Arrange
    let mut wheel = WheelSteps::default();

    // Act / Assert
    assert_eq!(wheel.due(), None);
    assert_eq!(wheel.take(Instant::now()), None);
}

#[test]
fn taking_the_steps_forgets_the_turns() {
    // Arrange
    let (mut wheel, last) = turned(&[1.0], Instant::now(), FAST);
    wheel.take(last + WHEEL_QUIET);

    // Act / Assert
    assert_eq!(wheel.due(), None);
    assert_eq!(wheel.take(last + WHEEL_QUIET * 2), None);
}

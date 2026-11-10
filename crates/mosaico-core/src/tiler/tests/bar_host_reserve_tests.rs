//! `[bar.reserve]` and the space a switch to another program's bar holds
//! until that bar says how far it reaches.

use std::time::Instant;

use crate::bar::HANDOVER;
use crate::config::{BarChoice, BarSection};

use super::bar_host_tests::{Recorder, host, host_on, picked};
use super::fake_platform::{Call, FakePlatform};
use super::manager_support::{start, still};

/// Another program's bar, with `top` pixels kept down from the top.
fn external_keeping(top: i32) -> BarSection {
    let mut section = picked(BarChoice::External);
    section.reserve.top = top;
    section
}

#[test]
fn another_programs_bar_keeps_the_reserve_whether_it_runs_or_not() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host_on(&Recorder::default(), BarChoice::Off);
    bars.start(&mut manager);

    // Act
    let changed = bars.choose(&mut manager, &external_keeping(40), Instant::now());

    // Assert
    assert!(changed);
    assert_eq!(manager.tiler().monitors()[0].work_area().y, 40);
}

#[test]
fn a_switch_to_another_programs_bar_holds_the_old_space_until_the_hold_is_over() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host(&Recorder::default());
    bars.start(&mut manager);
    platform.take_calls();
    let now = Instant::now();

    // Act
    bars.choose(&mut manager, &picked(BarChoice::External), now);
    let held = platform.take_calls();
    bars.wake(&mut manager, now + HANDOVER / 2);
    let early = manager.tiler().monitors()[0].work_area().y;
    bars.wake(&mut manager, now + HANDOVER);

    // Assert -- no window moves until the hold is over, then they take the
    // space at once.
    let placed = |calls: &[Call]| calls.iter().any(|c| matches!(c, Call::Place(..)));
    assert!(!placed(&held), "{held:?}");
    assert_eq!(early, 30);
    assert_eq!(manager.tiler().monitors()[0].work_area().y, 0);
    assert!(placed(&platform.take_calls()));
}

#[test]
fn space_the_new_bar_keeps_on_a_monitor_ends_the_hold_there() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host(&Recorder::default());
    bars.start(&mut manager);
    let now = Instant::now();
    bars.choose(&mut manager, &picked(BarChoice::External), now);

    // Act
    let changed = bars.choose(&mut manager, &external_keeping(20), now);

    // Assert -- a change to the reserve alone is not a change of bar.
    assert!(!changed);
    assert_eq!(manager.tiler().monitors()[0].work_area().y, 20);
}

#[test]
fn a_reserve_that_keeps_the_same_space_moves_no_window() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host_on(&Recorder::default(), BarChoice::Mosaico);
    bars.start(&mut manager);
    platform.take_calls();

    // Act -- mosaico's own bar keeps its own space whatever the reserve.
    let mut section = picked(BarChoice::Mosaico);
    section.reserve.top = 40;
    bars.choose(&mut manager, &section, Instant::now());

    // Assert
    let calls = platform.take_calls();
    assert!(calls.is_empty(), "{calls:?}");
}

#[test]
fn the_bars_are_due_when_the_hold_on_the_old_space_ends() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let mut manager = start(&platform, None, &still());
    let mut bars = host(&Recorder::default());
    bars.start(&mut manager);
    let now = Instant::now();

    // Act
    bars.choose(&mut manager, &picked(BarChoice::External), now);
    let held = bars.due();
    bars.wake(&mut manager, now + HANDOVER);

    // Assert -- nothing is due once the hold is over.
    assert_eq!(held, Some(now + HANDOVER));
    assert_eq!(bars.due(), None);
}

use super::*;
use crate::bar::fakes::FakeSystem;

/// A machine whose processors have spent `idle` of `total` time idle.
fn machine(idle: u64, total: u64) -> FakeSystem {
    let system = FakeSystem::default();
    advance(&system, idle, total);
    system
}

/// Moves the machine's processor counters on to `idle` of `total`.
fn advance(system: &FakeSystem, idle: u64, total: u64) {
    system.set(|m| m.cpu = Some(CpuTimes { idle, total }));
}

#[test]
fn the_first_sample_covers_the_time_since_the_tracker_started() {
    // Arrange -- 60 of the next 100 units are idle.
    let system = machine(100, 1000);
    let mut tracker = CpuTracker::new(&system);
    advance(&system, 160, 1100);

    // Act
    let usage = tracker.sample(&system);

    // Assert
    assert_eq!(usage, 40);
}

#[test]
fn each_sample_covers_the_time_since_the_previous_one() {
    // Arrange -- half idle, then 10 of the next 100 units idle.
    let system = machine(0, 0);
    let mut tracker = CpuTracker::new(&system);
    advance(&system, 50, 100);
    tracker.sample(&system);
    advance(&system, 60, 200);

    // Act
    let usage = tracker.sample(&system);

    // Assert
    assert_eq!(usage, 90);
}

#[test]
fn processors_that_never_idle_report_one_hundred() {
    // Arrange
    let system = machine(0, 0);
    let mut tracker = CpuTracker::new(&system);
    advance(&system, 0, 100);

    // Act
    let usage = tracker.sample(&system);

    // Assert
    assert_eq!(usage, 100);
}

#[test]
fn no_time_passing_reports_zero() {
    // Arrange
    let system = machine(100, 1000);
    let mut tracker = CpuTracker::new(&system);

    // Act
    let usage = tracker.sample(&system);

    // Assert
    assert_eq!(usage, 0);
}

#[test]
fn counters_that_go_back_report_zero() {
    // Arrange
    let system = machine(500, 1000);
    let mut tracker = CpuTracker::new(&system);
    advance(&system, 100, 200);

    // Act
    let usage = tracker.sample(&system);

    // Assert
    assert_eq!(usage, 0);
}

#[test]
fn more_idle_time_than_total_time_reports_zero() {
    // Arrange -- breaks the contract, but must not take the bar down.
    let system = machine(0, 0);
    let mut tracker = CpuTracker::new(&system);
    advance(&system, 200, 100);

    // Act
    let usage = tracker.sample(&system);

    // Assert
    assert_eq!(usage, 0);
}

#[test]
fn a_failed_read_reports_zero_and_the_next_sample_covers_the_gap() {
    // Arrange
    let system = machine(0, 0);
    let mut tracker = CpuTracker::new(&system);
    system.set(|m| m.cpu = None);
    let failed = tracker.sample(&system);
    advance(&system, 25, 100);

    // Act
    let usage = tracker.sample(&system);

    // Assert
    assert_eq!(failed, 0);
    assert_eq!(usage, 75);
}

#[test]
fn a_tracker_that_could_not_start_reports_zero_until_it_has_a_reading() {
    // Arrange
    let system = FakeSystem::default();
    let mut tracker = CpuTracker::new(&system);
    advance(&system, 100, 1000);
    let first = tracker.sample(&system);
    advance(&system, 150, 1100);

    // Act
    let second = tracker.sample(&system);

    // Assert
    assert_eq!(first, 0);
    assert_eq!(second, 50);
}

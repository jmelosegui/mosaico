use crate::WindowId;
use crate::id::wid;

use super::HoverTracker;

/// Feeds polls, as raw window ids, to a fresh tracker and returns what
/// it reported after each one.
fn rest_on(polls: &[Option<u64>]) -> Vec<Option<u64>> {
    let mut tracker = HoverTracker::default();
    polls
        .iter()
        .map(|poll| tracker.observe(poll.map(wid)).map(WindowId::raw))
        .collect()
}

#[test]
fn a_window_is_reported_once_the_pointer_rests_on_it() {
    // Act
    let reported = rest_on(&[Some(1), Some(1), Some(1)]);

    // Assert
    assert_eq!(reported, vec![None, None, Some(1)]);
}

#[test]
fn resting_on_a_window_reports_it_only_once() {
    // Act
    let reported = rest_on(&[Some(1); 6]);

    // Assert
    assert_eq!(reported, vec![None, None, Some(1), None, None, None]);
}

#[test]
fn moving_to_another_window_starts_the_count_again() {
    // Act
    let reported = rest_on(&[Some(1), Some(1), Some(2), Some(2), Some(2)]);

    // Assert
    assert_eq!(reported, vec![None, None, None, None, Some(2)]);
}

#[test]
fn a_poll_over_no_window_neither_counts_nor_breaks_a_rest() {
    // Act
    let unbroken = rest_on(&[Some(1), Some(1), None, Some(1)]);
    let uncounted = rest_on(&[Some(1), None, None]);

    // Assert
    assert_eq!(unbroken, vec![None, None, None, Some(1)]);
    assert_eq!(uncounted, vec![None, None, None]);
}

#[test]
fn coming_back_to_a_window_reports_it_again() {
    // Arrange
    let there_and_back = [1, 1, 1, 2, 2, 2, 1, 1, 1].map(Some);

    // Act
    let reported = rest_on(&there_and_back);

    // Assert
    let reports: Vec<u64> = reported.into_iter().flatten().collect();
    assert_eq!(reports, vec![1, 2, 1]);
}

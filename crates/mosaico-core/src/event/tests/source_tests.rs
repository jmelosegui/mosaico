use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::Error;

use super::EventWatch;

/// A watch whose platform stop counts how often it runs.
fn counting() -> (EventWatch, Arc<AtomicUsize>) {
    let stops = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&stops);
    let watch = EventWatch::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
    });
    (watch, stops)
}

#[test]
fn a_watch_leaves_the_source_running_until_it_is_stopped() {
    // Act
    let (watch, stops) = counting();

    // Assert
    assert_eq!(stops.load(Ordering::SeqCst), 0);
    drop(watch);
}

#[test]
fn stopping_a_watch_stops_the_source_once() {
    // Arrange
    let (watch, stops) = counting();

    // Act
    watch.stop();

    // Assert
    assert_eq!(stops.load(Ordering::SeqCst), 1);
}

#[test]
fn dropping_a_watch_stops_the_source() {
    // Arrange
    let (watch, stops) = counting();

    // Act
    drop(watch);

    // Assert
    assert_eq!(stops.load(Ordering::SeqCst), 1);
}

#[test]
fn a_watch_that_sees_the_displays_has_no_display_error() {
    // Act
    let (watch, _stops) = counting();

    // Assert
    assert!(watch.display_error().is_none());
}

#[test]
fn a_watch_without_the_displays_says_why_and_still_stops() {
    // Arrange
    let (watch, stops) = counting();

    // Act
    let watch = watch.without_displays(Error::Unsupported("watching displays"));

    // Assert
    assert!(matches!(
        watch.display_error(),
        Some(Error::Unsupported("watching displays"))
    ));
    watch.stop();
    assert_eq!(stops.load(Ordering::SeqCst), 1);
}

use std::cell::Cell;

use super::support::*;
use crate::ipc::{ResponseStatus, Subscribers};

#[test]
fn nobody_subscribed_means_no_snapshot_is_built() {
    // Arrange
    let mut publisher = Publisher::new(Subscribers::default());
    let built = Cell::new(false);

    // Act
    publisher.after_step(false, || {
        built.set(true);
        desktop()
    });

    // Assert
    assert!(!built.get());
}

#[test]
fn a_subscription_is_answered_with_the_desktop() {
    // Arrange
    let mut publisher = Publisher::new(Subscribers::default());

    // Act
    let response = publisher.subscribe(Vec::new(), desktop());

    // Assert
    assert_eq!(response.status, ResponseStatus::Ok);
    assert_eq!(response.data::<DesktopSnapshot>(), Some(desktop()));
}

#[test]
fn a_step_while_subscribed_compares_with_the_last_desktop() {
    // Arrange
    let subscribers = Subscribers::default();
    let mut publisher = Publisher::new(subscribers.clone());
    publisher.subscribe(Vec::new(), desktop());
    let built = Cell::new(0);

    // Act
    publisher.after_step(false, || {
        built.set(built.get() + 1);
        desktop()
    });
    publisher.close();

    // Assert
    assert_eq!(built.get(), 1);
    assert!(subscribers.is_empty());
}

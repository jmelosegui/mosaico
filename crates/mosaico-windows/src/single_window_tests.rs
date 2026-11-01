use std::sync::mpsc;
use std::time::Duration;

use super::*;

/// How long a test waits for a summons that should arrive.
const SUMMONS: Duration = Duration::from_secs(5);

/// A named instance of this test process, so its lock is the test's own
/// and never the user's settings window's.
fn test_instance(test: &str) -> Instance {
    let name = format!("test-{test}-{}", std::process::id());
    Instance::new(Some(&name), "cfg", "data").unwrap()
}

fn ignore() -> Summoned {
    Box::new(|| {})
}

#[test]
fn a_second_acquire_is_rejected_and_summons_the_first() {
    // Arrange
    let instance = test_instance("summons");
    let (tx, summons) = mpsc::channel();
    let summoned: Summoned = Box::new(move || {
        let _ = tx.send(());
    });
    let _first = NamedWindowLock
        .acquire(&instance, "settings", summoned)
        .expect("the first acquire should succeed");

    // Act
    let second = NamedWindowLock.acquire(&instance, "settings", ignore());

    // Assert
    assert!(matches!(second, Err(Error::AlreadyRunning)), "{second:?}");
    assert_eq!(summons.recv_timeout(SUMMONS), Ok(()));
}

#[test]
fn every_later_launch_summons_the_first_again() {
    // Arrange
    let instance = test_instance("again");
    let (tx, summons) = mpsc::channel();
    let summoned: Summoned = Box::new(move || {
        let _ = tx.send(());
    });
    let _first = NamedWindowLock
        .acquire(&instance, "settings", summoned)
        .unwrap();

    // Act / Assert
    for _ in 0..2 {
        let _ = NamedWindowLock.acquire(&instance, "settings", ignore());
        assert_eq!(summons.recv_timeout(SUMMONS), Ok(()));
    }
}

#[test]
fn other_apps_and_instances_lock_on_their_own() {
    // Arrange
    let instance = test_instance("apart");
    let _first = NamedWindowLock
        .acquire(&instance, "settings", ignore())
        .unwrap();

    // Act
    let other_app = NamedWindowLock.acquire(&instance, "other", ignore());
    let other_instance = NamedWindowLock.acquire(&test_instance("apart-2"), "settings", ignore());

    // Assert
    assert!(other_app.is_ok(), "{other_app:?}");
    assert!(other_instance.is_ok(), "{other_instance:?}");
}

#[test]
fn dropping_the_lock_lets_the_next_launch_take_it() {
    // Arrange
    let instance = test_instance("drop");
    let first = NamedWindowLock
        .acquire(&instance, "settings", ignore())
        .unwrap();

    // Act
    drop(first);
    let next = NamedWindowLock.acquire(&instance, "settings", ignore());

    // Assert
    assert!(next.is_ok(), "{next:?}");
}

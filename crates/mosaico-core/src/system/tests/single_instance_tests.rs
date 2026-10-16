use std::cell::Cell;
use std::rc::Rc;

use super::*;

#[test]
fn a_lock_is_released_once_when_it_is_dropped_and_not_before() {
    // Arrange
    let releases = Rc::new(Cell::new(0));
    let counter = Rc::clone(&releases);
    let lock = InstanceLock::new(move || counter.set(counter.get() + 1));

    // Act
    let while_held = releases.get();
    drop(lock);

    // Assert
    assert_eq!(while_held, 0, "released while still held");
    assert_eq!(releases.get(), 1);
}

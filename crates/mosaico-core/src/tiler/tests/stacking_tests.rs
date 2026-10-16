//! The stacking order the fake desktop keeps, as focusing a window and
//! putting it behind another change it.

use super::*;

/// A desktop with windows 1 and 2 open, 2 opened last, so in front.
fn desktop() -> FakeDesktop {
    let mut desktop = FakeDesktop::default();
    for id in [1, 2] {
        desktop.windows.insert(wid(id), FakeWindow::app(0));
        desktop.raise(wid(id));
    }
    desktop
}

#[test]
fn putting_a_window_behind_another_leaves_the_other_in_front() {
    // Arrange
    let mut desktop = desktop();
    assert!(desktop.in_front(wid(2), wid(1)));

    // Act
    desktop.apply(&Effect::PutBehind {
        window: wid(2),
        other: wid(1),
    });

    // Assert -- and nothing else changed.
    assert!(desktop.in_front(wid(1), wid(2)));
    assert_eq!(desktop.focused, None);
}

#[test]
fn a_window_put_behind_one_that_is_gone_stays_where_it_was() {
    // Arrange
    let mut desktop = desktop();

    // Act
    desktop.apply(&Effect::PutBehind {
        window: wid(2),
        other: wid(9),
    });

    // Assert
    assert!(desktop.in_front(wid(2), wid(1)));
}

#[test]
fn focusing_a_window_brings_it_to_the_front() {
    // Arrange
    let mut desktop = desktop();

    // Act
    desktop.apply(&Effect::Focus {
        window: wid(1),
        move_cursor: false,
    });

    // Assert
    assert!(desktop.in_front(wid(1), wid(2)));
}

use crate::WindowEvent;
use crate::id::wid;

use super::*;

fn moved(window: u64) -> WindowEvent {
    WindowEvent::LocationChanged {
        window: wid(window),
    }
}

fn retitled(window: u64) -> WindowEvent {
    WindowEvent::TitleChanged {
        window: wid(window),
    }
}

fn focused(window: u64) -> WindowEvent {
    WindowEvent::Focused {
        window: wid(window),
    }
}

#[test]
fn only_the_last_position_change_of_a_window_is_kept() {
    // Arrange
    let mut events = vec![moved(1), moved(1), moved(1)];

    // Act
    dedup(&mut events);

    // Assert
    assert_eq!(events, vec![moved(1)]);
}

#[test]
fn a_kept_change_stays_where_the_last_one_arrived() {
    // Arrange
    let mut events = vec![moved(1), focused(2), moved(1)];

    // Act
    dedup(&mut events);

    // Assert
    assert_eq!(events, vec![focused(2), moved(1)]);
}

#[test]
fn position_and_title_changes_are_collapsed_separately() {
    // Arrange
    let mut events = vec![retitled(1), moved(1), retitled(1), moved(1)];

    // Act
    dedup(&mut events);

    // Assert
    assert_eq!(events, vec![retitled(1), moved(1)]);
}

#[test]
fn changes_of_different_windows_are_all_kept() {
    // Arrange
    let mut events = vec![moved(1), moved(2), retitled(1), retitled(2)];
    let expected = events.clone();

    // Act
    dedup(&mut events);

    // Assert
    assert_eq!(events, expected);
}

#[test]
fn other_events_are_never_dropped_or_reordered() {
    // Arrange -- repeats included, since handling them is not idempotent.
    let mut events = vec![
        focused(1),
        WindowEvent::Created { window: wid(2) },
        focused(1),
        WindowEvent::Moved { window: wid(2) },
        WindowEvent::Minimized { window: wid(1) },
        WindowEvent::Restored { window: wid(1) },
        WindowEvent::MouseHover { window: wid(2) },
        WindowEvent::Hidden { window: wid(3) },
        WindowEvent::Destroyed { window: wid(2) },
        WindowEvent::WorkAreaChanged,
        WindowEvent::DisplayChanged,
        WindowEvent::DisplayChanged,
    ];
    let expected = events.clone();

    // Act
    dedup(&mut events);

    // Assert
    assert_eq!(events, expected);
}

use crate::WindowEvent;
use crate::event::DragStart;
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

fn grabbed(window: u64) -> WindowEvent {
    WindowEvent::MoveStarted {
        window: wid(window),
        start: DragStart::default(),
    }
}

fn dropped(window: u64) -> WindowEvent {
    WindowEvent::Moved {
        window: wid(window),
        pointer: Some((10, 20)),
    }
}

#[test]
fn position_changes_are_not_collapsed_across_the_start_of_a_drag() {
    // Arrange
    let mut events = vec![moved(1), grabbed(1), moved(1), moved(1)];

    // Act
    dedup(&mut events);

    // Assert -- the last one before the drag stays before it.
    assert_eq!(events, vec![moved(1), grabbed(1), moved(1)]);
}

#[test]
fn position_changes_are_not_collapsed_across_the_end_of_a_drag() {
    // Arrange
    let mut events = vec![grabbed(1), moved(1), dropped(1), moved(1)];
    let expected = events.clone();

    // Act
    dedup(&mut events);

    // Assert -- the drag keeps where the window was while it lasted.
    assert_eq!(events, expected);
}

#[test]
fn the_drag_of_one_window_keeps_the_changes_of_every_window_apart() {
    // Arrange
    let mut events = vec![moved(2), retitled(2), grabbed(1), moved(2), retitled(2)];
    let expected = events.clone();

    // Act
    dedup(&mut events);

    // Assert
    assert_eq!(events, expected);
}

#[test]
fn changes_while_a_window_is_dragged_are_still_collapsed() {
    // Arrange -- the near miss: nothing between them is a barrier.
    let mut events = vec![
        grabbed(1),
        moved(1),
        retitled(1),
        moved(1),
        retitled(1),
        moved(1),
        dropped(1),
    ];

    // Act
    dedup(&mut events);

    // Assert
    assert_eq!(events, vec![grabbed(1), retitled(1), moved(1), dropped(1)]);
}

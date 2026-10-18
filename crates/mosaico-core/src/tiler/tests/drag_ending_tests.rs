use crate::Rect;
use crate::event::{Edges, Grab};
use crate::id::wid;

use super::super::Drag;
use super::*;

/// Window 1, dragged from monitor 0 by `grab`, from its slot.
fn drag(grab: Option<Grab>) -> Drag {
    Drag {
        window: wid(1),
        monitor: 0,
        grab,
        from_maximized: false,
        start: Rect::new(10, 40, 480, 750),
        pointer: Some((250, 50)),
        left_start: true,
        decides: true,
        preview: None,
    }
}

#[test]
fn a_drag_by_an_edge_resizes_and_one_by_the_title_bar_moves() {
    // Arrange
    let edge = Grab::Edges(Edges {
        left: true,
        top: true,
        ..Edges::default()
    });
    let moved = Rect::new(300, 140, 480, 750);

    // Act
    let by_edge = ending(&drag(Some(edge)), moved, false, true);
    let by_title = ending(&drag(Some(Grab::TitleBar)), moved, false, true);

    // Assert
    assert_eq!(by_edge, Ending::Resized);
    assert_eq!(by_title, Ending::Moved);
}

#[test]
fn without_a_grab_a_drag_that_changed_the_size_resized() {
    // Act
    let resized = ending(&drag(None), Rect::new(10, 40, 600, 750), false, true);
    let moved = ending(&drag(None), Rect::new(300, 140, 480, 750), false, true);

    // Assert
    assert_eq!(resized, Ending::Resized);
    assert_eq!(moved, Ending::Moved);
}

#[test]
fn a_move_the_system_snapped_or_maximized_was_snapped() {
    // Arrange -- half the monitor, or all of it, maximized.
    let drag = drag(Some(Grab::TitleBar));
    let half = Rect::new(0, 30, 500, 770);

    // Act
    let snapped = ending(&drag, half, false, true);
    let maximized = ending(&drag, Rect::new(0, 30, 1000, 770), true, true);

    // Assert
    assert_eq!(snapped, Ending::Snapped);
    assert_eq!(maximized, Ending::Snapped);
}

#[test]
fn a_move_the_system_maximized_on_another_monitor_was_moved() {
    // Arrange -- whatever the platform says was grabbed.
    let all = Rect::new(1000, 30, 1000, 770);

    for grab in [Some(Grab::TitleBar), None] {
        // Act
        let ended = ending(&drag(grab), all, true, false);

        // Assert -- there, as one snapped there.
        assert_eq!(ended, Ending::Moved, "{grab:?}");
    }
}

#[test]
fn a_move_that_changed_the_size_by_a_few_pixels_on_its_monitor_was_not_snapped() {
    // Arrange -- a pixel wider, as a monitor of another scale rounds it,
    // up to four on an edge, and then one more.
    let drag = drag(Some(Grab::TitleBar));
    let wider = |by: i32| Rect::new(300, 140, 480 + by, 750 - by);

    // Act
    let ended = [1, 4, 5].map(|by| ending(&drag, wider(by), false, true));

    // Assert
    assert_eq!(ended, [Ending::Moved, Ending::Moved, Ending::Snapped]);
}

#[test]
fn a_move_rescaled_on_another_monitor_is_still_a_move() {
    // Arrange -- the near miss of a snap: the system scaled it up as it
    // crossed to a monitor of another scale.
    let drag = drag(Some(Grab::TitleBar));

    // Act
    let ended = ending(&drag, Rect::new(1100, 60, 720, 1125), false, false);

    // Assert
    assert_eq!(ended, Ending::Moved);
}

#[test]
fn a_maximized_window_dragged_out_of_it_was_moved() {
    // Arrange -- it shrinks back to its normal size under the pointer.
    let drag = Drag {
        from_maximized: true,
        grab: Some(Grab::TitleBar),
        start: Rect::new(0, 30, 1000, 770),
        ..drag(None)
    };

    // Act
    let ended = ending(&drag, Rect::new(400, 100, 480, 750), false, true);

    // Assert
    assert_eq!(ended, Ending::Moved);
}

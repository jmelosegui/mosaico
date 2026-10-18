//! What a pass of the daemon loop hands the tiler while a window is
//! dragged: one report of where it is, the latest, however many it made,
//! so the tiles that follow the drag are laid out once a pass.

use crate::event::DragStart;

use super::*;

/// The user starts dragging window `window`.
fn grabbed(window: u64) -> WindowEvent {
    WindowEvent::MoveStarted {
        window: wid(window),
        start: DragStart::default(),
    }
}

#[test]
fn a_pass_hands_on_one_report_of_a_dragged_window_however_many_it_made() {
    // Arrange -- window 1 reports where it is at every pixel of a resize,
    // more often than the loop gets to it, and window 2 once, as a tile
    // that follows it does.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let mut batch = vec![event(grabbed(1))];
    batch.extend((0..20).map(|_| event(moved(1))));
    batch.push(event(moved(2)));
    batch.extend((0..20).map(|_| event(moved(1))));

    // Act
    let steps = scheduler.step(received(batch), t0);

    // Assert
    let handed = vec![grabbed(1), moved(2), moved(1)];
    assert_eq!(steps, vec![Step::Events(handed)]);
}

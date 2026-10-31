//! The keys on the focused floating window: move nudges it within its
//! monitor, and the resize keys grow and shrink it around its center.

use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::{Action, Direction, Rect, WindowEvent};

/// `monitors` monitors with tiles 1 and 2 on the first, window 2 floated
/// with focus.
fn floated(monitors: usize) -> Harness {
    let mut h = Harness::new(monitors, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);
    h
}

#[test]
fn moving_a_floating_window_nudges_it_and_leaves_the_tiles() {
    // Arrange
    let mut h = floated(1);
    let before = h.rect_of(2);
    let step = h.tiler.monitors()[0].work_area().width / 20;

    // Act
    h.action(Action::Move(Direction::Right));

    // Assert
    assert_eq!(
        h.rect_of(2),
        Rect {
            x: before.x + step,
            ..before
        }
    );
    assert_eq!(h.active(0), vec![1]);
}

/// Nudges the focused floating window 2 `dir` until it touches the left or
/// right edge of monitor `mon`.
fn nudge_to_edge(h: &mut Harness, mon: usize, dir: Direction) {
    let area = h.tiler.monitors()[mon].work_area();
    let at_edge = |r: Rect| match dir {
        Direction::Left => r.x == area.x,
        _ => r.x + r.width == area.x + area.width,
    };
    while !at_edge(h.rect_of(2)) {
        h.action(Action::Move(dir));
    }
}

#[test]
fn moving_a_floating_window_stops_at_the_edge_of_its_monitor() {
    // Arrange
    let mut h = floated(2);
    nudge_to_edge(&mut h, 0, Direction::Right);
    let at_edge = h.rect_of(2);

    // Act
    h.action(Action::Move(Direction::Right));

    // Assert
    assert_eq!(h.workspace(0, 0).floating(), &[wid(2)]);
    assert!(h.workspace(1, 0).floating().is_empty());
    assert_eq!(h.rect_of(2), at_edge);
}

#[test]
fn the_resize_keys_grow_and_shrink_a_floating_window_around_its_center() {
    // Arrange
    let mut h = floated(1);
    let before = h.rect_of(2);

    // Act
    h.action(Action::GrowWidth);
    let wider = h.rect_of(2);
    h.action(Action::ShrinkHeight);

    // Assert
    assert!(wider.width > before.width);
    assert_eq!(wider.center_x(), before.center_x());
    assert!(h.rect_of(2).height < before.height);
}

#[test]
fn the_resize_keys_do_nothing_on_a_tile() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::Focused { window: wid(1) });

    // Act
    h.action(Action::GrowWidth);

    // Assert
    assert!(h.placements().is_empty());
}

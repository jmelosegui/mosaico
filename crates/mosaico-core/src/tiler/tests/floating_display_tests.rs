//! Floating windows when a monitor goes away and comes back.

use crate::id::wid;
use crate::tiler::tests::{Harness, displays};
use crate::tiler::{BarReservation, Input};
use crate::{Action, WindowEvent};

fn connect(h: &mut Harness, count: usize) {
    h.step(Input::Displays {
        monitors: displays(count),
        bar: BarReservation::default(),
    });
}

/// Window 1 on the primary, windows 2 and 3 on the second monitor, with
/// window 3 floated there.
fn floated_on_second() -> Harness {
    let mut h = Harness::new(2, &[(1, 0), (2, 1), (3, 1)]);
    h.event(WindowEvent::Focused { window: wid(3) });
    h.action(Action::ToggleFloat);
    h
}

#[test]
fn a_float_whose_monitor_goes_away_floats_in_view_on_the_one_left() {
    // Arrange
    let mut h = floated_on_second();
    let rect = h.rect_of(3);

    // Act
    connect(&mut h, 1);

    // Assert -- at the same size, in the same place on the other monitor.
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.workspace(0, 0).floating(), &[wid(3)]);
    let area = h.tiler.monitors()[0].work_area();
    assert_eq!(
        h.rect_of(3),
        crate::Rect {
            x: rect.x - area.width,
            ..rect
        }
    );
    assert!(h.window(3).visible && !h.window(3).cloaked);
}

#[test]
fn a_float_goes_back_to_its_monitor_when_it_comes_back() {
    // Arrange
    let mut h = floated_on_second();
    let rect = h.rect_of(3);
    connect(&mut h, 1);

    // Act
    connect(&mut h, 2);

    // Assert
    assert_eq!(h.active(1), vec![2]);
    assert!(h.workspace(0, 0).floating().is_empty());
    assert_eq!(h.workspace(1, 0).floating(), &[wid(3)]);
    assert_eq!(h.rect_of(3), rect);
}

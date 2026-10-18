//! The border of a window the user drags through the end of its drag, as
//! the owner chose while testing: on the window through Escape and a key,
//! and where it was until the drop when no report comes.

use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::{Action, Direction, WindowEvent};

use super::super::support::grab;
use super::tests::border_at;

#[test]
fn escape_and_a_key_leave_the_border_on_the_window() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let start = h.rect_of(1);
    grab(&mut h, 1);

    // Act -- a key interrupts the drag, the window goes on 50 pixels, and
    // the system puts it back where it started for Escape.
    h.action(Action::Focus(Direction::Left));
    h.window(1).rect.x += 50;
    let on = h.event(WindowEvent::LocationChanged { window: wid(1) });
    let interrupted = border_at(on, 1);
    let further = h.rect_of(1);
    h.window(1).rect = start;
    let back = h.event(WindowEvent::LocationChanged { window: wid(1) });

    // Assert
    assert_eq!(interrupted, Some(further));
    assert_eq!(border_at(back, 1), Some(start));
}

#[test]
fn without_reports_the_border_stays_where_it_was_until_the_drop() {
    // Arrange -- the user grabs window 1, which has focus.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let rect = h.rect_of(1);
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(rect),
        ..DragStart::default()
    };
    let started = h
        .event(WindowEvent::MoveStarted {
            window: wid(1),
            start,
        })
        .to_vec();

    // Act -- it moves with window contents hidden while dragging, which
    // reports nothing until the drop.
    h.window(1).rect.x += 300;
    let dropped = h.event(WindowEvent::moved(wid(1))).to_vec();

    // Assert -- nothing moves the border before the drop puts it back
    // around the window in its slot.
    assert_eq!(border_at(&started, 1), None);
    assert_eq!(border_at(&dropped, 1), Some(rect));
}

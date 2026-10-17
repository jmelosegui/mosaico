//! Edge drags that end as no resize: a press let go within a few pixels of
//! where it was, which is a click however the window reported its size,
//! and a window left maximized as it is dropped, which stays so.

use crate::id::wid;
use crate::tiler::tests::{Harness, MON_H, MON_W};
use crate::{Rect, WindowEvent};

use super::support::{RIGHT, grab_to, held, holding, resizing, unmoved};

/// Windows 1 and 2 side by side, with splits moved by the edges of the
/// windows the user resizes.
fn pair() -> Harness {
    Harness::with_settings(resizing(), 1, &[(1, 0), (2, 0)])
}

#[test]
fn a_press_on_an_edge_let_go_where_it_was_is_a_click() {
    // Arrange -- window 1 grabbed by its right edge, and reported 8 pixels
    // wider, as an application that sizes in steps does.
    let mut h = pair();
    let left = h.rect_of(1);
    let pressed = holding(left, RIGHT);
    let wider = Rect {
        width: left.width + 8,
        ..left
    };
    grab_to(&mut h, 1, RIGHT, wider);

    // Act -- let go two pixels from where the pointer pressed.
    let pointer = Some((pressed.0 + 2, pressed.1));
    h.event(WindowEvent::Moved {
        window: wid(1),
        pointer,
    });

    // Assert -- back in its slot.
    assert!(unmoved(&h));
    assert_eq!(h.rect_of(1), left);
}

#[test]
fn a_window_left_maximized_as_an_edge_drag_ends_stays_maximized() {
    // Arrange -- window 1 grabbed by its top right corner.
    let mut h = pair();
    let corner = held(false, true, true, false);
    let left = h.rect_of(1);
    let dragged = Rect::new(left.x, left.y - 4, left.width + 300, left.height + 4);
    grab_to(&mut h, 1, corner, dragged);

    // Act -- the system maximizes it as it is dropped.
    let full = Rect::new(0, 0, MON_W, MON_H);
    h.window(1).rect = full;
    h.window(1).maximized = true;
    let pointer = Some(holding(full, corner));
    h.event(WindowEvent::Moved {
        window: wid(1),
        pointer,
    });

    // Assert -- no split moves, and nothing places it.
    assert!(unmoved(&h));
    assert!(h.window(1).maximized);
    assert_eq!(h.rect_of(1), full);
}

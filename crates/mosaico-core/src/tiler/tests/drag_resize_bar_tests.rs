//! Resizes and the bars: a click or the wheel on a bar while the user
//! resizes a window interrupts the resize, as a key does, and a resize let
//! go with the pointer over a bar moves its split all the same, to where
//! the window's edge is.

use crate::action::Origin;
use crate::id::wid;
use crate::tiler::Settings;
use crate::tiler::tests::Harness;
use crate::{Action, Rect, WindowEvent};

use super::super::support::reserve_bar;
use super::support::{RIGHT, grab_to, held, holding, resizing, unmoved};

#[test]
fn a_click_or_the_wheel_on_a_bar_during_a_resize_leaves_its_end_as_before() {
    // The wheel over the second monitor's bar, a click on one of its
    // workspaces, and a click on the layout widget of the first.
    let inputs = [
        (Action::GoToWorkspaceNext, Origin::Bar(1)),
        (Action::GoToWorkspace(2), Origin::Bar(1)),
        (Action::CycleLayout, Origin::Bar(0)),
    ];
    for (action, origin) in inputs {
        // Arrange -- window 1 resized by its right edge, still held.
        let mut h = Harness::with_settings(resizing(), 2, &[(1, 0), (2, 0), (3, 1)]);
        let left = h.rect_of(1);
        let wider = Rect {
            width: left.width + 300,
            ..left
        };
        grab_to(&mut h, 1, RIGHT, wider);

        // Act
        h.action_from(action, origin);
        let pointer = Some(holding(wider, RIGHT));
        h.event(WindowEvent::Moved {
            window: wid(1),
            pointer,
        });

        // Assert -- back in its slot, and no split moved.
        let slot = h.tiler.slot_of(0, wid(1));
        assert_eq!(Some(h.rect_of(1)), slot, "{action:?}");
        assert_ne!(h.rect_of(1), wider, "{action:?}");
        assert!(unmoved(&h), "{action:?}");
    }
}

#[test]
fn a_resize_let_go_over_a_bar_moves_its_split_to_where_the_window_is() {
    // Arrange -- a bar at the top, and window 2 above window 3, held by its
    // bottom edge, which its minimum size stops 200 pixels below its top
    // as the pointer goes on up over the bar.
    let settings = Settings {
        drag_to_swap: true,
        ..resizing()
    };
    let mut h = Harness::with_settings(settings, 1, &[(1, 0), (2, 0), (3, 0)]);
    reserve_bar(&mut h);
    let above = h.rect_of(2);
    let shorter = Rect {
        height: 200,
        ..above
    };
    grab_to(&mut h, 2, held(false, false, false, true), shorter);

    // Act
    let pointer = Some((above.center_x(), 10));
    h.event(WindowEvent::Moved {
        window: wid(2),
        pointer,
    });

    // Assert -- the split between windows 2 and 3 is where the edge of
    // window 2 was dropped, not where the pointer is.
    assert_eq!(h.rect_of(2), shorter);
    assert!(!unmoved(&h));
}

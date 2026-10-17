//! The border of a window the user drags: drawn until the window leaves
//! where it started, hidden from then until the drag ends.

use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{Effect, Input};
use crate::{WindowEvent, WindowId};

use super::super::support::grab;

/// Whether `effects` hide the border of `window`.
fn hide(effects: &[Effect], window: WindowId) -> bool {
    let hides = |e: &Effect| matches!(e, Effect::Border { window: w, style: None } if *w == window);
    effects.iter().any(hides)
}

#[test]
fn the_border_of_a_dragged_window_hides_once_it_leaves_where_it_started() {
    // Arrange -- window 1 has focus and its border.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let rect = h.rect_of(1);
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(rect),
        ..DragStart::default()
    };

    // Act -- the user grabs it, then it reports where it is as it moves.
    let started = h
        .event(WindowEvent::MoveStarted {
            window: wid(1),
            start,
        })
        .to_vec();
    h.window(1).rect.x += 20;
    let moved = h
        .event(WindowEvent::LocationChanged { window: wid(1) })
        .to_vec();

    // Assert
    assert!(!hide(&started, wid(1)), "{started:?}");
    assert!(hide(&moved, wid(1)), "{moved:?}");
    assert_eq!(h.desktop.borders.get(&wid(1)), Some(&false));
}

#[test]
fn the_border_of_a_dragged_window_stays_hidden_until_the_drag_ends() {
    // Arrange -- window 1 has focus and its border.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    grab(&mut h, 1);
    let hidden = h.desktop.borders.get(&wid(1)).copied();

    // Act -- every border is drawn again while it is dragged, then the
    // drag ends.
    h.step(Input::RefreshBorders);
    let dragging = h.desktop.borders.get(&wid(1)).copied();
    h.event(WindowEvent::moved(wid(1)));

    // Assert
    assert_eq!(hidden, Some(false));
    assert_eq!(dragging, Some(false));
    assert_eq!(h.desktop.borders.get(&wid(1)), Some(&true));
}

#[test]
fn a_click_on_the_title_bar_leaves_the_border_drawn() {
    // Arrange -- the platform reports a press and a release on the title
    // bar of window 1, which has focus, as a drag that moves nothing.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let rect = h.rect_of(1);
    let at = Some((rect.center_x(), rect.y + 10));
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(rect),
        maximized: Some(false),
        pointer: at,
    };

    // Act -- in between, the window reports where it is, where it was,
    // and every border is drawn again.
    let mut effects = h
        .event(WindowEvent::MoveStarted {
            window: wid(1),
            start,
        })
        .to_vec();
    effects.extend_from_slice(h.event(WindowEvent::LocationChanged { window: wid(1) }));
    effects.extend_from_slice(h.step(Input::RefreshBorders));
    let (window, pointer) = (wid(1), at);
    effects.extend_from_slice(h.event(WindowEvent::Moved { window, pointer }));

    // Assert -- its border is never hidden.
    assert!(!hide(&effects, wid(1)), "{effects:?}");
    assert_eq!(h.desktop.borders.get(&wid(1)), Some(&true));
}

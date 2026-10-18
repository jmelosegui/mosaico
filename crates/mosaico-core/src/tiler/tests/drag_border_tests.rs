//! The border of a window the user drags, as the owner chose while
//! testing: it stays on the window, drawn where each report has it, a move
//! or a resize, through every border drawn again, and a click on the title
//! bar leaves it drawn.

use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{Effect, Input};
use crate::{Rect, WindowEvent, WindowId};

use super::super::resize::support::{RIGHT, grab_to, resizing};
use super::super::support::grab;

/// Whether `effects` hide the border of `window`.
fn hide(effects: &[Effect], window: WindowId) -> bool {
    let hides = |e: &Effect| matches!(e, Effect::Border { window: w, style: None } if *w == window);
    effects.iter().any(hides)
}

/// Where `effects` draw the border of window `id` last, if they draw it.
pub(super) fn border_at(effects: &[Effect], id: u64) -> Option<Rect> {
    effects.iter().rev().find_map(|e| match e {
        Effect::Border { window, style } if *window == wid(id) => style.as_ref().map(|s| s.rect),
        _ => None,
    })
}

#[test]
fn the_border_of_a_dragged_window_follows_it() {
    // Arrange -- window 1 has focus and its border.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);

    // Act -- grabbed by its title bar, it reports it is 100 pixels further.
    grab(&mut h, 1);

    // Assert -- its border is drawn there, not hidden.
    let moved = h.rect_of(1);
    assert_eq!(border_at(&h.effects, 1), Some(moved));
    assert!(!hide(&h.effects, wid(1)), "{:?}", h.effects);
    assert_eq!(h.desktop.borders.get(&wid(1)), Some(&true));
}

#[test]
fn the_border_of_a_window_resized_by_an_edge_takes_its_size() {
    // Arrange
    let mut h = Harness::with_settings(resizing(), 1, &[(1, 0), (2, 0)]);
    let left = h.rect_of(1);

    // Act -- 200 pixels wider, still held.
    let wider = Rect {
        width: left.width + 200,
        ..left
    };
    grab_to(&mut h, 1, RIGHT, wider);

    // Assert
    assert_eq!(border_at(&h.effects, 1), Some(wider));
}

#[test]
fn the_border_of_a_dragged_window_stays_on_it_until_the_drag_ends() {
    // Arrange -- window 1 has focus and its border.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let slot = h.rect_of(1);
    grab(&mut h, 1);
    let moved = h.rect_of(1);

    // Act -- every border is drawn again while it is dragged, then the
    // drag ends.
    let refreshed = h.step(Input::RefreshBorders).to_vec();
    let dropped = h.event(WindowEvent::moved(wid(1))).to_vec();

    // Assert -- around the window, wherever it is.
    assert_eq!(border_at(&refreshed, 1), Some(moved));
    assert_eq!(border_at(&dropped, 1), Some(slot));
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

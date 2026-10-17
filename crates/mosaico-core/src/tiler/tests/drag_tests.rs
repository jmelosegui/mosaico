//! The drags the tiler follows, and what it leaves alone meanwhile.

use crate::id::wid;
use crate::tiler::Effect;
use crate::tiler::tests::fake_desktop::FakeWindow;
use crate::tiler::tests::{Harness, MON_W};
use crate::{Action, WindowEvent};

use super::support::{grab, places};

#[test]
fn a_window_opening_while_one_is_dragged_is_tiled_without_moving_the_dragged_one() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    grab(&mut h, 1);
    let held = h.rect_of(1);

    // Act
    let effects = h.open(3, 0).to_vec();

    // Assert -- it keeps its slot in the layout, and nothing places it.
    assert_eq!(h.active(0), vec![1, 2, 3]);
    assert!(places(&effects, 2) && places(&effects, 3), "{effects:?}");
    assert!(!places(&effects, 1), "{effects:?}");
    assert_eq!(h.rect_of(1), held);
}

#[test]
fn a_monocle_window_dragged_while_another_closes_is_not_placed() {
    // Arrange -- window 1 fills the monitor in monocle, window 2 behind.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.action(Action::ToggleMonocle);
    grab(&mut h, 1);

    // Act -- window 2 closes, and the workspace is laid out again.
    h.desktop.windows.remove(&wid(2));
    let effects = h.event(WindowEvent::Destroyed { window: wid(2) }).to_vec();

    // Assert
    assert_eq!(h.active(0), vec![1]);
    assert!(!places(&effects, 1), "{effects:?}");
}

#[test]
fn a_dragged_window_coming_out_of_maximized_is_not_put_back_in_its_slot() {
    // Arrange -- window 1, focused, is maximized, and the user drags it
    // out of it, over the second monitor.
    let mut h = Harness::new(2, &[(1, 0), (2, 0), (3, 1)]);
    h.window(1).maximized = true;
    h.event(WindowEvent::LocationChanged { window: wid(1) });
    grab(&mut h, 1);
    h.window(1).maximized = false;
    h.window(1).rect.x += MON_W;
    h.window(1).monitor = 1;

    // Act
    let effects = h
        .event(WindowEvent::LocationChanged { window: wid(1) })
        .to_vec();

    // Assert -- as it comes out of maximized under the pointer, it stays
    // in its workspace, and nothing is laid out around it.
    let placed = |e: &Effect| matches!(e, Effect::Place { .. });
    assert!(!effects.iter().any(placed), "{effects:?}");
    assert_eq!((h.active(0), h.active(1)), (vec![1, 2], vec![3]));
}

#[test]
fn the_drag_of_a_window_the_tiler_leaves_alone_is_not_followed() {
    // Arrange -- a window no rule manages, over the tiles.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let mut floating = FakeWindow::app(0);
    floating.facts = None;
    h.desktop.windows.insert(wid(9), floating);

    // Act
    let effects = h.event(WindowEvent::move_started(wid(9))).to_vec();
    let opening = h.open(3, 0).to_vec();

    // Assert -- nothing changes, and the tiles go on being laid out.
    assert!(effects.is_empty(), "{effects:?}");
    assert!(h.tiler.mouse.drag.is_none());
    assert!(places(&opening, 1) && places(&opening, 2), "{opening:?}");
}

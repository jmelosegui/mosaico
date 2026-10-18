//! Dragging tiled windows with the mouse on the live desktop.
//!
//! Like `window_management.rs`, these tests start a daemon of their own,
//! launch notepad and tile the live desktop, and they press, move and
//! release the real pointer, so they run on demand, on the desktop runner,
//! never with the safe suites. Two notepads side by side on an empty
//! workspace are dragged by their title bar, or resized by the edge between
//! them, which the right one follows while the edge is held, with
//! transitions off so each window jumps straight to where it lands.

#[path = "drag/caption.rs"]
mod caption;
mod common;
#[path = "transitions/notepads.rs"]
mod notepads;
#[path = "drag/pointer.rs"]
mod pointer;
#[path = "transitions/premise.rs"]
mod premise;
#[path = "drag/win32.rs"]
mod win32;

use std::thread;
use std::time::Duration;

use mosaico_core::system::ProcessSetup;
use mosaico_core::{Rect, WindowId};
use mosaico_windows::setup::WindowsSetup;

use common::{RunningDaemon, TestInstance, desktop_guard};
use notepads::Notepads;

/// The gap the daemon leaves between two windows, with the default config.
const GAP: i32 = 8;

/// Where a drag grabs the left notepad, from its window and its visible
/// bounds.
type GrabAt = fn(WindowId, Rect) -> (i32, i32);

/// Where a drag drops the left notepad, from where the pointer grabbed it
/// and the two notepads' visible bounds, left first.
type DropAt = fn((i32, i32), Rect, Rect) -> (i32, i32);

/// Where the two notepads are, left first: before a drag, while the
/// pointer still holds the left one where it drops it, and once they are
/// still after the drop.
type Dragged = ([Rect; 2], [Rect; 2], [Rect; 2]);

/// Opens two notepads side by side and drags the left one by its title bar,
/// as [`drag_left`] does.
fn drag_left_to(label: &str, drop: DropAt) -> Dragged {
    drag_left(label, caption::title_bar, drop)
}

/// Where the pointer grabs the left notepad by its right edge, once a click
/// on its title bar, which changes nothing, has brought it in front of the
/// right one, whose frame reaches over the gap between them.
fn in_front_by_right_edge(window: WindowId, rect: Rect) -> (i32, i32) {
    let title = caption::title_bar(window, rect);
    pointer::drag(title, title);
    caption::right_edge(window, rect)
}

/// Opens two notepads side by side on workspace 8, on a daemon with
/// transitions off, then grabs the left one where `grab` says and drops it
/// with the pointer where `drop` says. Returns where the notepads are, as
/// [`Dragged`] says.
fn drag_left(label: &str, grab: GrabAt, drop: DropAt) -> Dragged {
    let instance = TestInstance::new(label);
    instance.write_config("config.toml", "[animations]\nenabled = false\n");
    // Read and move the pointer in the physical pixels the daemon works in.
    WindowsSetup.init();
    instance.start_daemon();
    let _daemon = RunningDaemon::new(&instance);
    instance.run(&["action", "go-to-workspace", "8"]);
    thread::sleep(Duration::from_millis(500));

    let mut notepads = Notepads::new();
    let mut windows: [WindowId; 2] = [notepads.open(), notepads.open()];
    let mut before = notepads.still(windows);
    let state = instance.stdout(&["debug", "state"]);
    premise::assert_alone_side_by_side(&state, windows, before);
    if before[0].x > before[1].x {
        windows.swap(0, 1);
        before.swap(0, 1);
    }

    let [left, right] = before;
    let grabbed = grab(windows[0], left);
    let mut held = before;
    let to = drop(grabbed, left, right);
    pointer::drag_holding(grabbed, to, || held = notepads.still(windows));
    (before, held, notepads.still(windows))
}

#[test]
fn a_window_dropped_on_another_swaps_places_with_it() {
    // Arrange
    let _guard = desktop_guard();

    // Act -- onto the middle of the right notepad.
    let (before, _, after) = drag_left_to("drop-swap", |_, _, right| {
        (right.center_x(), right.center_y())
    });

    // Assert -- each ends exactly in the other's slot.
    assert_eq!(after, [before[1], before[0]]);
}

#[test]
fn a_window_dropped_on_its_own_slot_goes_back_to_it() {
    // Arrange -- the near miss: a drag that stays over its own slot.
    let _guard = desktop_guard();

    // Act -- a little down and to the left of where it was grabbed.
    let (before, _, after) = drag_left_to("drop-back", |(x, y), _, _| (x - 80, y + 120));

    // Assert -- both are where they were.
    assert_eq!(after, before);
}

#[test]
fn a_window_resized_by_the_edge_between_two_moves_the_split() {
    // Arrange
    let _guard = desktop_guard();

    // Act -- the left notepad's right edge, 200 pixels right.
    let (before, held, after) =
        drag_left("resize-split", in_front_by_right_edge, |(x, y), _, _| {
            (x + 200, y)
        });

    // Assert -- while the edge is still held, the right notepad starts
    // one gap after it already, as window contents shown while dragging
    // report it; once let go, the edge stays where it was dropped, to the
    // few pixels a press may wobble, and the right notepad still starts
    // one gap after it.
    let [left, right] = held;
    assert_eq!(right.x, left.x + left.width + GAP, "{before:?} -> {held:?}");
    let [left, right] = after;
    let moved = (left.x + left.width) - (before[0].x + before[0].width);
    assert!((196..=204).contains(&moved), "{before:?} -> {after:?}");
    assert_eq!(
        right.x,
        left.x + left.width + GAP,
        "{before:?} -> {after:?}"
    );
}

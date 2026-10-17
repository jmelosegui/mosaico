//! What the drag tests share: the user grabbing, dragging and dropping a
//! window, what a step places, and a bar on every monitor.

use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::tests::{Harness, MON_W};
use crate::tiler::{BarReservation, Effect, Input, Settings};
use crate::{Rect, WindowEvent};

/// Settings with windows dropped on tiles moving there.
pub(super) fn swapping() -> Settings {
    Settings {
        drag_to_swap: true,
        ..Settings::default()
    }
}

/// Reserves a 30 pixel bar at the top of every monitor of `h`.
pub(super) fn reserve_bar(h: &mut Harness) {
    h.step(Input::Bar(BarReservation {
        height: 30,
        monitors: Vec::new(),
    }));
}

/// Whether `effects` place window `id`.
pub(super) fn places(effects: &[Effect], id: u64) -> bool {
    let placed = |e: &Effect| matches!(*e, Effect::Place { window, .. } if window == wid(id));
    effects.iter().any(placed)
}

/// The user grabs window `id` by its title bar where it is, which gives
/// it focus, and moves it 100 pixels to the right, which the window
/// reports.
pub(super) fn grab(h: &mut Harness, id: u64) {
    let rect = h.rect_of(id);
    h.event(WindowEvent::Focused { window: wid(id) });
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(rect),
        maximized: Some(h.desktop.windows[&wid(id)].maximized),
        pointer: Some(title_bar(rect)),
    };
    h.event(WindowEvent::MoveStarted {
        window: wid(id),
        start,
    });
    move_to(h, id, (title_bar(rect).0 + 100, title_bar(rect).1));
    h.event(WindowEvent::LocationChanged { window: wid(id) });
}

/// The user drags window `id` by its title bar, the window following the
/// pointer, and drops it with the pointer at `to`. Returns what the drop
/// does.
pub(super) fn drag_to(h: &mut Harness, id: u64, to: (i32, i32)) -> Vec<Effect> {
    grab(h, id);
    move_to(h, id, to);
    drop_at(h, id, to)
}

/// The window dragged with its title bar under the pointer at `to`, and
/// on the monitor that holds its center.
pub(super) fn move_to(h: &mut Harness, id: u64, to: (i32, i32)) {
    let window = h.window(id);
    let (x, y) = title_bar(window.rect);
    window.rect.x += to.0 - x;
    window.rect.y += to.1 - y;
    window.monitor = (window.rect.center_x().max(0) / MON_W) as usize;
}

/// The drag of window `id` ends with the pointer at `to`. Returns what the
/// drop does.
pub(super) fn drop_at(h: &mut Harness, id: u64, to: (i32, i32)) -> Vec<Effect> {
    let window = wid(id);
    let pointer = Some(to);
    h.event(WindowEvent::Moved { window, pointer }).to_vec()
}

/// Where the pointer holds a window by its title bar.
pub(super) fn title_bar(rect: Rect) -> (i32, i32) {
    (rect.center_x(), rect.y + 10)
}

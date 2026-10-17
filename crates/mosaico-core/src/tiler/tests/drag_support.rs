//! What the drag tests share: the user grabbing a window, and what a step
//! places.

use crate::WindowEvent;
use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::Effect;
use crate::tiler::tests::Harness;

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
        pointer: Some((rect.center_x(), rect.y + 10)),
    };
    h.event(WindowEvent::MoveStarted {
        window: wid(id),
        start,
    });
    h.window(id).rect.x += 100;
    h.event(WindowEvent::LocationChanged { window: wid(id) });
}

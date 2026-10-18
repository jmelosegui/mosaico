//! What dragging a tiled window and dropping it asks of the platform: onto
//! another tile to swap them, and onto another monitor; and what resizing
//! one by the edge between two tiles does.

use crate::WindowEvent;
use crate::event::{DragStart, Edges, Grab};
use crate::id::wid;
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::{start, still};
use crate::tiler::tests::metrics_support::{Manager, Measured, four, on};

/// Four windows on one monitor; window 1, the left half, is resized by its
/// right edge, which window 2 is beyond, 200 pixels to the right, and let
/// go there.
pub(super) fn resize() -> Measured {
    let arranged = four();
    let platform = arranged.0.clone();
    on(arranged, |manager| {
        let window = wid(1);
        let rect = platform.window(1).rect;
        let edge = Edges {
            right: true,
            ..Edges::default()
        };
        let at = |rect: crate::Rect| (rect.x + rect.width - 1, rect.center_y());
        manager.handle_event(&WindowEvent::Focused { window });
        let start = DragStart {
            grab: Some(Grab::Edges(edge)),
            rect: Some(rect),
            maximized: Some(false),
            pointer: Some(at(rect)),
        };
        manager.handle_event(&WindowEvent::MoveStarted { window, start });
        platform.edit(1, |w| w.rect.width += 200);
        manager.handle_event(&WindowEvent::LocationChanged { window });
        let pointer = Some(at(platform.window(1).rect));
        manager.handle_event(&WindowEvent::Moved { window, pointer });
    })
}

/// Four windows on one monitor; window 1 is dragged onto window 2 and
/// dropped there.
pub(super) fn to_swap() -> Measured {
    let arranged = four();
    let platform = arranged.0.clone();
    let over = platform.window(2).rect;
    let to = (over.center_x(), over.center_y());
    on(arranged, |manager| drag(&platform, manager, 1, to))
}

/// Two windows on each of two monitors; window 1 is dragged onto window
/// 4, on the second, and dropped there.
pub(super) fn to_another_monitor() -> Measured {
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);
    let manager = start(&platform, None, &still());
    let over = platform.window(4).rect;
    let to = (over.center_x(), over.center_y());
    let dragged = platform.clone();
    on((platform, manager), |manager| {
        drag(&dragged, manager, 1, to)
    })
}

/// The user grabs window `id` by its title bar, which gives it focus,
/// drags it so that the pointer is at `to`, with one position reported on
/// the way, and drops it there.
fn drag(platform: &FakePlatform, manager: &mut Manager, id: u64, to: (i32, i32)) {
    let window = wid(id);
    let rect = platform.window(id).rect;
    let from = (rect.center_x(), rect.y + 10);
    manager.handle_event(&WindowEvent::Focused { window });
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(rect),
        maximized: Some(false),
        pointer: Some(from),
    };
    manager.handle_event(&WindowEvent::MoveStarted { window, start });
    platform.edit(id, |w| {
        w.rect.x += to.0 - from.0;
        w.rect.y += to.1 - from.1;
        w.monitor = usize::from(to.0 >= crate::tiler::tests::MON_W);
    });
    platform.point_at(to);
    manager.handle_event(&WindowEvent::LocationChanged { window });
    let pointer = Some(to);
    manager.handle_event(&WindowEvent::Moved { window, pointer });
}

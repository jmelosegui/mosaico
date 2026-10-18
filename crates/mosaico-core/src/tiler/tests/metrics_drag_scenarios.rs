//! What dragging a tiled window and dropping it asks of the platform: onto
//! another tile to swap them, and onto another monitor, over one tile and
//! over three places in turn; and what resizing one by the edge between
//! two tiles does, with one report on the way and with several.

use crate::WindowEvent;
use crate::event::{DragStart, Edges, Grab};
use crate::id::wid;
use crate::tiler::tests::MON_W;
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::{settle, start, still};
use crate::tiler::tests::metrics_support::{Manager, Measured, four, on};

/// Four windows on one monitor; window 1, the left half, is resized by its
/// right edge, which window 2 is beyond, 200 pixels to the right, and let
/// go there.
pub(super) fn resize() -> Measured {
    resize_in(&[200])
}

/// As [`resize`], with the edge reported five times on its way, 40
/// pixels apart, each moving the tiles beyond it.
pub(super) fn resize_followed() -> Measured {
    resize_in(&[40; 5])
}

/// Four windows on one monitor; window 1 is resized by its right edge, as
/// far as each of `steps` takes it in turn, reporting each, and let go.
fn resize_in(steps: &[i32]) -> Measured {
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
        for &step in steps {
            platform.edit(1, |w| w.rect.width += step);
            manager.handle_event(&WindowEvent::LocationChanged { window });
        }
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
    on(arranged, |manager| {
        drag(&platform, manager, 1, &[to], false)
    })
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
        drag(&dragged, manager, 1, &[to], false)
    })
}

/// Two windows on each of two monitors; window 1 is dragged over window 3
/// on the second, on over window 4, and on to the gap above the tiles
/// there, where it would join the end, and dropped there: the target
/// changes twice, and the tiles there make room each time.
pub(super) fn over_another_monitor() -> Measured {
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);
    let manager = start(&platform, None, &still());
    let path = path_over(&platform);
    let dragged = platform.clone();
    on((platform, manager), |manager| {
        drag(&dragged, manager, 1, &path, false)
    })
}

/// Where the pointer goes as window 1 is dragged over another monitor:
/// over window 3, over window 4, and to the gap above them.
pub(super) fn path_over(platform: &FakePlatform) -> [(i32, i32); 3] {
    let center = |id: u64| {
        let rect = platform.window(id).rect;
        (rect.center_x(), rect.center_y())
    };
    [center(3), center(4), (MON_W + MON_W / 2, 2)]
}

/// The user grabs window `id` by its title bar, which gives it focus,
/// drags it so that the pointer is at each point of `path` in turn, with a
/// position reported at each, the windows let settle after each when
/// `settling`, and drops it at the last.
pub(super) fn drag(
    platform: &FakePlatform,
    manager: &mut Manager,
    id: u64,
    path: &[(i32, i32)],
    settling: bool,
) {
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
    for &to in path {
        platform.edit(id, |w| {
            w.rect.x = rect.x + to.0 - from.0;
            w.rect.y = rect.y + to.1 - from.1;
            w.monitor = usize::from(to.0 >= MON_W);
        });
        platform.point_at(to);
        manager.handle_event(&WindowEvent::LocationChanged { window });
        if settling {
            settle(platform, manager);
        }
    }
    let pointer = path.last().copied();
    manager.handle_event(&WindowEvent::Moved { window, pointer });
}

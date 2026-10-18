//! What the tests of the tiles that follow a drag on the tiling manager
//! share: windows on two monitors with transitions on, the user dragging
//! one by its title bar, its reports and its drop.

use crate::WindowEvent;
use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::TilingManager;
use crate::tiler::tests::MON_W;
use crate::tiler::tests::fake_platform::{Call, FakePlatform};
use crate::tiler::tests::manager_support::{animated, settle, start};

pub(super) type Manager = TilingManager<FakePlatform>;

/// Windows 1 and 2 on the first monitor and 3 and 4 on the second, managed
/// with transitions on, once they have settled.
pub(super) fn two_monitors() -> (FakePlatform, Manager) {
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);
    let mut manager = start(&platform, None, &animated());
    settle(&platform, &mut manager);
    (platform, manager)
}

/// The user grabs window `id` by its title bar, which gives it focus.
pub(super) fn grab(platform: &FakePlatform, manager: &mut Manager, id: u64) {
    let rect = platform.window(id).rect;
    manager.handle_event(&WindowEvent::Focused { window: wid(id) });
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(rect),
        maximized: Some(false),
        pointer: Some((rect.center_x(), rect.y + 10)),
    };
    manager.handle_event(&WindowEvent::MoveStarted {
        window: wid(id),
        start,
    });
}

/// Window `id`, which the user drags, follows the pointer to `to`, with
/// its title bar under it, and reports it. Returns whether any window
/// moves over time then.
pub(super) fn reported(
    platform: &FakePlatform,
    manager: &mut Manager,
    id: u64,
    to: (i32, i32),
) -> bool {
    platform.edit(id, |w| {
        w.rect.x = to.0 - w.rect.width / 2;
        w.rect.y = to.1 - 10;
        w.monitor = usize::from(to.0 >= MON_W);
    });
    platform.point_at(to);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(id) });
    manager.wants_frames()
}

/// As [`reported`], and the windows get to what was asked of them.
/// Returns what that asked of the platform too.
pub(super) fn report_at(
    platform: &FakePlatform,
    manager: &mut Manager,
    id: u64,
    to: (i32, i32),
) -> (Vec<Call>, bool) {
    platform.take_calls();
    let moving = reported(platform, manager, id, to);
    settle(platform, manager);
    (platform.take_calls(), moving)
}

/// The drag of window `id` ends with the pointer at `to`, and the windows
/// settle. Returns what that asked of the platform.
pub(super) fn drop_at(
    platform: &FakePlatform,
    manager: &mut Manager,
    id: u64,
    to: (i32, i32),
) -> Vec<Call> {
    let pointer = Some(to);
    manager.handle_event(&WindowEvent::Moved {
        window: wid(id),
        pointer,
    });
    settle(platform, manager);
    platform.take_calls()
}

/// Whether `calls` move window `id`, by a placement or a frame.
pub(super) fn moves(calls: &[Call], id: u64) -> bool {
    calls.iter().any(|call| match call {
        Call::Place(window, _) => *window == wid(id),
        Call::PlaceFrames(frames) => frames.iter().any(|f| f.window == wid(id)),
        _ => false,
    })
}

/// The center of where window `id` is.
pub(super) fn center(platform: &FakePlatform, id: u64) -> (i32, i32) {
    let rect = platform.window(id).rect;
    (rect.center_x(), rect.center_y())
}

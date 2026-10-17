//! Drags and drops on the tiling manager, with transitions on: the window
//! the user drags never moves on its own, what the drop moves animates,
//! and a display change noted mid-drag leaves the drop as before.

use crate::WindowEvent;
use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::TilingManager;
use crate::tiler::tests::fake_platform::{Call, FakePlatform};
use crate::tiler::tests::manager_support::{animated, settle, start, tiled};

type Manager = TilingManager<FakePlatform>;

/// The user grabs window `id` by its title bar, which gives it focus, and
/// drags it so that the pointer is at `to`.
fn drag(platform: &FakePlatform, manager: &mut Manager, id: u64, to: (i32, i32)) {
    let rect = platform.window(id).rect;
    let from = (rect.center_x(), rect.y + 10);
    manager.handle_event(&WindowEvent::Focused { window: wid(id) });
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(rect),
        maximized: Some(false),
        pointer: Some(from),
    };
    manager.handle_event(&WindowEvent::MoveStarted {
        window: wid(id),
        start,
    });
    platform.edit(id, |w| {
        w.rect.x += to.0 - from.0;
        w.rect.y += to.1 - from.1;
    });
}

/// Whether `calls` move window `id`, by a placement or a frame.
fn moves(calls: &[Call], id: u64) -> bool {
    calls.iter().any(|call| match call {
        Call::Place(window, _) => *window == wid(id),
        Call::PlaceFrames(frames) => frames.iter().any(|f| f.window == wid(id)),
        _ => false,
    })
}

#[test]
fn a_window_dragged_while_the_others_move_is_left_where_the_user_has_it() {
    // Arrange -- the user drags window 1 a little.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    platform.edit(3, |w| w.visible = false);
    let mut manager = start(&platform, None, &animated());
    settle(&platform, &mut manager);
    let at = platform.window(1).rect;
    drag(&platform, &mut manager, 1, (at.center_x() + 50, at.y + 60));
    let held = platform.window(1).rect;
    platform.take_calls();

    // Act -- window 3 opens meanwhile, and the others make room.
    platform.edit(3, |w| w.visible = true);
    manager.handle_event(&WindowEvent::Created { window: wid(3) });
    manager.flush_pending();
    settle(&platform, &mut manager);

    // Assert
    let calls = platform.take_calls();
    assert!(moves(&calls, 2), "{calls:?}");
    assert!(!moves(&calls, 1), "{calls:?}");
    assert_eq!(platform.window(1).rect, held);
}

#[test]
fn a_drop_that_swaps_two_windows_moves_them_into_their_slots_over_time() {
    // Arrange -- window 1 dragged onto window 2.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &animated());
    settle(&platform, &mut manager);
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    let over = (right.center_x(), right.center_y());
    drag(&platform, &mut manager, 1, over);

    // Act
    let pointer = Some(over);
    manager.handle_event(&WindowEvent::Moved {
        window: wid(1),
        pointer,
    });
    let moving = manager.wants_frames();
    settle(&platform, &mut manager);

    // Assert -- both slide, and end in each other's slots.
    assert!(moving);
    assert_eq!(platform.window(1).rect, right);
    assert_eq!(platform.window(2).rect, left);
}

#[test]
fn a_display_change_noted_while_a_window_is_dragged_leaves_its_drop_as_before() {
    // Arrange -- window 1 dragged over window 2, then a display change is
    // noted.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &animated());
    settle(&platform, &mut manager);
    let right = platform.window(2).rect;
    let over = (right.center_x(), right.center_y());
    drag(&platform, &mut manager, 1, over);
    manager.display_changing();

    // Act -- dropped there before the monitors are read again.
    let pointer = Some(over);
    manager.handle_event(&WindowEvent::Moved {
        window: wid(1),
        pointer,
    });

    // Assert -- back to its slot, as a drop the tiler did not follow.
    assert_eq!(tiled(&manager, 0), vec![1, 2]);
}

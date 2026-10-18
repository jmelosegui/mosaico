//! Resizing tiled windows by their edges on the tiling manager, with
//! transitions on: the window the user resizes is never placed while they
//! hold it, the windows the new split moves follow it at once, sent where
//! it puts them once, and the drop moves only the window back into its
//! slot.

use crate::event::{DragStart, Edges, Grab};
use crate::id::wid;
use crate::tiler::TilingManager;
use crate::tiler::tests::fake_platform::{Call, FakePlatform};
use crate::tiler::tests::manager_support::{animated, settle, start};
use crate::{Action, Direction, WindowEvent};

type Manager = TilingManager<FakePlatform>;

/// The right edge of a window, held alone.
const RIGHT: Edges = Edges {
    left: false,
    top: false,
    right: true,
    bottom: false,
};

/// The user grabs window `id` by its right edge, which gives it focus, and
/// makes it 300 pixels wider, which it reports, without letting go.
fn widen(platform: &FakePlatform, manager: &mut Manager, id: u64) {
    let window = wid(id);
    let rect = platform.window(id).rect;
    manager.handle_event(&WindowEvent::Focused { window });
    let start = DragStart {
        grab: Some(Grab::Edges(RIGHT)),
        rect: Some(rect),
        maximized: Some(false),
        pointer: Some((rect.x + rect.width - 1, rect.center_y())),
    };
    manager.handle_event(&WindowEvent::MoveStarted { window, start });
    platform.edit(id, |w| w.rect.width += 300);
    manager.handle_event(&WindowEvent::LocationChanged { window });
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
fn a_window_resized_while_another_opens_is_left_where_the_user_has_it() {
    // Arrange -- the user holds window 1 by its edge.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    platform.edit(3, |w| w.visible = false);
    let mut manager = start(&platform, None, &animated());
    settle(&platform, &mut manager);
    widen(&platform, &mut manager, 1);
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
fn the_windows_a_resize_moves_follow_it_at_once_and_only_its_window_settles_at_the_drop() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &animated());
    settle(&platform, &mut manager);
    platform.take_calls();

    // Act -- 300 pixels wider, which window 2 follows, and let go with
    // the window 50 pixels shorter, its bottom edge an outer one.
    widen(&platform, &mut manager, 1);
    let followed = (platform.take_calls(), manager.wants_frames());
    let wider = platform.window(1).rect;
    platform.edit(1, |w| w.rect.height -= 50);
    let pointer = Some((wider.x + wider.width - 1, wider.center_y()));
    manager.handle_event(&WindowEvent::Moved {
        window: wid(1),
        pointer,
    });
    let moving = manager.wants_frames();
    settle(&platform, &mut manager);
    let dropped = platform.take_calls();

    // Assert -- window 2 is placed beside the edge at once, with no frame;
    // the drop leaves it there, and window 1 slides back to its height.
    assert!(moves(&followed.0, 2) && !followed.1, "{:?}", followed.0);
    assert!(moving);
    assert!(!moves(&dropped, 2), "{dropped:?}");
    assert!(moves(&dropped, 1), "{dropped:?}");
    assert_eq!(platform.window(1).rect, wider);
    assert_eq!(platform.window(2).rect.x, wider.x + wider.width + 8);
}

#[test]
fn a_tile_still_on_its_way_to_where_a_resize_put_it_is_not_sent_there_again() {
    // Arrange -- windows 1 and 2 swapped and back over time, so what is
    // sent to them may be on its way for a moment.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &animated());
    for direction in [Direction::Right, Direction::Left] {
        manager.handle_action(&Action::Move(direction));
        manager.flush_pending();
        settle(&platform, &mut manager);
    }
    platform.take_calls();

    // Act -- 300 pixels wider, reported twice before window 2 gets to
    // where the first report sent it.
    widen(&platform, &mut manager, 1);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });

    // Assert -- window 2 was sent there once.
    let calls = platform.take_calls();
    let second = |c: &&Call| matches!(c, Call::Place(window, _) if *window == wid(2));
    assert_eq!(calls.iter().filter(second).count(), 1, "{calls:?}");
}

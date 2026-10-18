//! Dragging a tiled window over another monitor in monocle on the tiling
//! manager, with transitions on: its windows make room at once and the
//! drop leaves them there, and one still on its way to where the drag put
//! it, as what is sent to a window that moved over time can be, is not
//! sent there again.

use crate::Action;
use crate::action::Origin;
use crate::id::wid;
use crate::tiler::drag::live::back::manager_support::{
    Manager, center, drop_at, grab, moves, report_at, reported, two_monitors,
};
use crate::tiler::tests::fake_platform::{Call, FakePlatform};
use crate::tiler::tests::manager_support::settle;

/// Windows 1 and 2 on the first monitor and 3 and 4 on the second, window
/// 3 in monocle there, from its bar, which it moved to fill over time.
fn monocle_on_second() -> (FakePlatform, Manager) {
    let (platform, mut manager) = two_monitors();
    manager.handle_action_from(&Action::ToggleMonocle, Origin::Bar(1));
    manager.flush_pending();
    settle(&platform, &mut manager);
    (platform, manager)
}

#[test]
fn a_monocle_workspace_there_makes_room_at_once_and_the_drop_leaves_its_windows_there() {
    // Arrange -- where window 4 is tiled, behind window 3.
    let (platform, mut manager) = monocle_on_second();
    let over = center(&platform, 4);
    grab(&platform, &mut manager, 1);

    // Act -- over there, and dropped there.
    let (made_room, moving) = report_at(&platform, &mut manager, 1, over);
    let dropped = drop_at(&platform, &mut manager, 1, over);

    // Assert
    assert!(
        moves(&made_room, 3) && moves(&made_room, 4),
        "{made_room:?}"
    );
    assert!(!moving);
    assert!(!moves(&dropped, 3) && !moves(&dropped, 4), "{dropped:?}");
    assert!(moves(&dropped, 1), "{dropped:?}");
}

#[test]
fn a_tile_still_on_its_way_to_where_the_drag_put_it_is_not_sent_there_again() {
    // Arrange
    let (platform, mut manager) = monocle_on_second();
    let (x, y) = center(&platform, 4);
    grab(&platform, &mut manager, 1);
    platform.take_calls();

    // Act -- two reports over where window 4 is tiled, before window 3
    // gets to where the first sent it.
    reported(&platform, &mut manager, 1, (x, y));
    reported(&platform, &mut manager, 1, (x + 5, y + 5));

    // Assert -- window 3 was sent its slot once.
    let calls = platform.take_calls();
    let third = |c: &&Call| matches!(c, Call::Place(window, _) if *window == wid(3));
    assert_eq!(calls.iter().filter(third).count(), 1, "{calls:?}");
}

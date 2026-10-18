//! Dragging a tiled window over another monitor on the tiling manager,
//! with transitions on: the tiles there make room at once, never moving
//! there over time, a drop that lands as they were laid out leaves them
//! where they are, and the tiles a drag leaves slide back.

use crate::tiler::drag::live::back::manager_support::{
    center, drop_at, grab, moves, report_at, two_monitors,
};
use crate::tiler::tests::manager_support::settle;

#[test]
fn the_tiles_of_another_monitor_make_room_at_once_and_the_drop_leaves_them_there() {
    // Arrange
    let (platform, mut manager) = two_monitors();
    let over = center(&platform, 3);
    grab(&platform, &mut manager, 1);

    // Act -- over window 3, and dropped there.
    let (made_room, moving) = report_at(&platform, &mut manager, 1, over);
    let dropped = drop_at(&platform, &mut manager, 1, over);

    // Assert -- windows 3 and 4 are placed with no frame; the drop moves
    // window 1 in, and window 2 into the room it left, but not them.
    assert!(
        moves(&made_room, 3) && moves(&made_room, 4),
        "{made_room:?}"
    );
    assert!(!moving);
    assert!(!moves(&dropped, 3) && !moves(&dropped, 4), "{dropped:?}");
    assert!(moves(&dropped, 1) && moves(&dropped, 2), "{dropped:?}");
}

#[test]
fn the_tiles_a_drag_leaves_slide_back_into_their_slots() {
    // Arrange -- over window 3.
    let (platform, mut manager) = two_monitors();
    let (home, over) = (center(&platform, 1), center(&platform, 3));
    let slots = (platform.window(3).rect, platform.window(4).rect);
    grab(&platform, &mut manager, 1);
    report_at(&platform, &mut manager, 1, over);

    // Act -- back over its own slot.
    let (_, moving) = report_at(&platform, &mut manager, 1, home);
    settle(&platform, &mut manager);

    // Assert
    assert!(moving);
    assert_eq!((platform.window(3).rect, platform.window(4).rect), slots);
}

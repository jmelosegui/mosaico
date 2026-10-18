//! The tiles that made room for a dragged window, sent back and still on
//! their way when the drag lays them out again: as the pointer comes back
//! over the same tile before they get back, and after a layout pass of
//! their monitor that keeps their slots (#160 review, round 1, finding 2).

use crate::tiler::drag::live::back::manager_support::{center, grab, reported, two_monitors};
use crate::tiler::tests::manager_support::{animated, settle};

#[test]
fn a_quick_return_over_the_same_tile_makes_room_again() {
    // Arrange -- over window 3, then home, so windows 3 and 4 start back.
    let (platform, mut manager) = two_monitors();
    let (home, over) = (center(&platform, 1), center(&platform, 3));
    grab(&platform, &mut manager, 1);
    reported(&platform, &mut manager, 1, over);
    settle(&platform, &mut manager);
    let made = (platform.window(3).rect, platform.window(4).rect);
    reported(&platform, &mut manager, 1, home);

    // Act -- over window 3 again, before a frame.
    reported(&platform, &mut manager, 1, over);
    settle(&platform, &mut manager);

    // Assert -- they make room again, as the first time.
    let (third, fourth) = (platform.window(3).rect, platform.window(4).rect);
    assert_eq!((third, fourth), made);
}

#[test]
fn a_layout_pass_meanwhile_does_not_leave_the_tiles_where_it_sent_them() {
    // Arrange -- over window 3, the tiles there in place.
    let (platform, mut manager) = two_monitors();
    let over = center(&platform, 3);
    grab(&platform, &mut manager, 1);
    reported(&platform, &mut manager, 1, over);
    settle(&platform, &mut manager);
    let made = (platform.window(3).rect, platform.window(4).rect);

    // Act -- the config read again, which lays every monitor out, and one
    // more report over window 3.
    manager.reload_config(&animated());
    reported(&platform, &mut manager, 1, over);
    settle(&platform, &mut manager);

    // Assert
    let (third, fourth) = (platform.window(3).rect, platform.window(4).rect);
    assert_eq!((third, fourth), made);
}

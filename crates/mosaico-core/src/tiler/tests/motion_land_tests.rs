//! Frames that do not leave a window where they were posted: absorbed, or
//! still queued when its motion ends, or when a display change places it.
//! The placement lands after them.

use super::display_tests::{fits, shorter};
use super::{REFRESH, animated, frame_by_frame, placements, settle, swap, two};

#[test]
fn a_window_that_absorbs_its_frames_is_still_placed_exactly_at_its_target() {
    // Arrange -- window 1 takes every frame without moving.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    platform.edit(1, |w| w.absorbs_frames = true);
    swap(&mut manager);

    // Act -- then its application gets to what is still queued for it.
    let calls = frame_by_frame(&platform, &mut manager).concat();
    platform.catch_up(1);

    // Assert -- its frames move it nowhere, its final placement does.
    assert!(placements(&calls).contains(&(1, right)), "{calls:?}");
    assert_eq!(platform.window(1).rect, right);
    assert_eq!(platform.window(2).rect, left);
}

#[test]
fn a_window_placed_while_frames_are_on_their_way_ends_where_it_was_placed() {
    // Arrange -- window 1's application takes three frames to get to each.
    let (platform, mut manager) = two(&animated());
    let right = platform.window(2).rect;
    platform.edit(1, |w| w.frame_delay = 3);
    swap(&mut manager);

    // Act -- its motion ends with frames still queued for it; then its
    // application gets to them.
    frame_by_frame(&platform, &mut manager);
    let queued = platform.window(1).queued.len();
    platform.catch_up(1);

    // Assert -- the placement lands after them, not before.
    assert!(queued > 1, "{queued} queued");
    assert_eq!(platform.window(1).rect, right);
}

#[test]
fn a_display_change_while_a_frame_is_on_its_way_places_the_windows_behind_it() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    platform.edit(1, |w| w.frame_delay = 3);
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();

    // Act -- then window 1's application gets to what is queued for it.
    manager.handle_display_change(shorter(), 0, &[]);
    settle(&platform, &mut manager);
    platform.catch_up(1);

    // Assert -- side by side in the smaller work area.
    let (one, two) = (platform.window(1).rect, platform.window(2).rect);
    assert!(two.x < one.x, "{two:?} {one:?}");
    assert!(fits(&one) && fits(&two), "{one:?} {two:?}");
}

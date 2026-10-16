//! Windows on their way on a platform without frames or with frames that
//! fail, and the window in front as monocle turns off.

use crate::id::wid;
use crate::{Action, Direction};

use super::{REFRESH, animated, frame_by_frame, placements, posted, settle, swap, two};

#[test]
fn a_platform_without_frames_places_every_window_at_once_from_then_on() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    platform.refuse_frames();
    swap(&mut manager);

    // Act -- the first frame finds out.
    platform.advance(REFRESH);
    manager.frame();
    let first = platform.take_calls();
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();

    // Assert -- the next swap asks for no frame at all.
    assert_eq!(placements(&first), [(2, left), (1, right)]);
    let next = platform.take_calls();
    assert_eq!(placements(&next), [(1, left), (2, right)]);
    assert!(posted(&next).is_empty());
    assert!(!manager.wants_frames());
}

#[test]
fn frames_that_fail_send_the_windows_on_their_way_to_their_slots_and_no_further() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    platform.break_frames(1);
    swap(&mut manager);

    // Act -- the first frame fails.
    platform.advance(REFRESH);
    manager.frame();
    let first = platform.take_calls();
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();

    // Assert -- placed at once, as nothing was posted to either, and the
    // next swap moves them frame by frame again.
    assert_eq!(placements(&first), [(2, left), (1, right)]);
    assert!(manager.wants_frames());
    let next = frame_by_frame(&platform, &mut manager).concat();
    assert!(!posted(&next).is_empty());
    assert_eq!(platform.window(1).rect, left);
}

#[test]
fn turning_monocle_off_shrinks_the_window_in_front_back_to_its_tile_alone() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    let tile = platform.window(1).rect;
    manager.handle_action(&Action::ToggleMonocle);
    settle(&platform, &mut manager);
    platform.take_calls();

    // Act
    manager.handle_action(&Action::ToggleMonocle);
    let calls = frame_by_frame(&platform, &mut manager).concat();

    // Assert -- window 2 never left its tile, and is not moved.
    let frames = posted(&calls);
    assert!(!frames.is_empty());
    assert!(frames.iter().flatten().all(|f| f.window == wid(1)));
    assert_eq!(placements(&calls), [(1, tile)]);
    assert_eq!(platform.window(1).rect, tile);
}

//! The tiles of another monitor that make room for a dragged window, on
//! the tiling manager with transitions on: they glide there as any layout
//! change does, turn from where they are as the target changes, glide
//! back as the drag leaves.

use std::time::Duration;

use crate::Rect;
use crate::id::wid;
use crate::tiler::drag::live::back::manager_support::{
    Manager, center, grab, reported, two_monitors,
};
use crate::tiler::tests::fake_platform::{Call, FakePlatform};
use crate::tiler::tests::manager_support::settle;

/// One frame step, a display refresh after the last.
pub(super) fn frame(platform: &FakePlatform, manager: &mut Manager) {
    platform.advance(Duration::from_millis(16));
    manager.frame();
}

/// Whether `calls` place window `id` at once.
pub(super) fn placed_at_once(calls: &[Call], id: u64) -> bool {
    calls
        .iter()
        .any(|call| matches!(call, Call::Place(window, _) if *window == wid(id)))
}

/// The rects window `id` goes through over `n` frame steps.
pub(super) fn rects(
    platform: &FakePlatform,
    manager: &mut Manager,
    id: u64,
    n: usize,
) -> Vec<Rect> {
    (0..n)
        .map(|_| {
            frame(platform, manager);
            platform.window(id).rect
        })
        .collect()
}

#[test]
fn the_tiles_of_another_monitor_glide_to_make_room() {
    // Arrange
    let (platform, mut manager) = two_monitors();
    let over = center(&platform, 3);
    grab(&platform, &mut manager, 1);
    platform.take_calls();

    // Act -- over window 3.
    let moving = reported(&platform, &mut manager, 1, over);
    let calls = platform.take_calls();
    settle(&platform, &mut manager);

    // Assert -- windows 3 and 4 move over time, not at once, to the top
    // and the bottom of the right half.
    assert!(moving);
    assert!(
        !placed_at_once(&calls, 3) && !placed_at_once(&calls, 4),
        "{calls:?}"
    );
    let right = platform.window(4).rect;
    assert_eq!(platform.window(3).rect.x, right.x);
    assert!(platform.window(3).rect.y < right.y);
}

#[test]
fn a_target_change_mid_glide_turns_the_tiles_from_where_they_are() {
    // Arrange -- over window 3, three frames into the glide.
    let (platform, mut manager) = two_monitors();
    let (third, fourth) = (center(&platform, 3), center(&platform, 4));
    let left = platform.window(3).rect;
    grab(&platform, &mut manager, 1);
    reported(&platform, &mut manager, 1, third);
    let mid = *rects(&platform, &mut manager, 3, 3).last().expect("frames");

    // Act -- on to window 4, which sends window 3 back to the left half.
    reported(&platform, &mut manager, 1, fourth);
    let next = rects(&platform, &mut manager, 3, 1)[0];
    settle(&platform, &mut manager);

    // Assert -- it turns back from where it was, and ends there.
    assert!(left.x < mid.x, "{mid:?}");
    assert!(left.x < next.x && next.x < mid.x, "{mid:?} -> {next:?}");
    assert_eq!(platform.window(3).rect, left);
}

#[test]
fn the_tiles_glide_back_as_the_drag_leaves_mid_glide() {
    // Arrange -- over window 3, three frames into the glide.
    let (platform, mut manager) = two_monitors();
    let (home, over) = (center(&platform, 1), center(&platform, 3));
    let slots = (platform.window(3).rect, platform.window(4).rect);
    grab(&platform, &mut manager, 1);
    reported(&platform, &mut manager, 1, over);
    let mid = *rects(&platform, &mut manager, 3, 3).last().expect("frames");

    // Act -- back over its own slot.
    reported(&platform, &mut manager, 1, home);
    let next = rects(&platform, &mut manager, 3, 1)[0];
    settle(&platform, &mut manager);

    // Assert
    assert!(slots.0.x < next.x && next.x < mid.x, "{mid:?} -> {next:?}");
    assert_eq!((platform.window(3).rect, platform.window(4).rect), slots);
}

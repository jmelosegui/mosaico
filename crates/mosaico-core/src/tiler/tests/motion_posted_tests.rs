//! Windows that what they were last sent may still be on its way to, once
//! their motions are over: the desktop may still have them where they
//! were, so their placements are not skipped for it, but are for one in
//! place that was last sent there, and the pointer goes where they are
//! going.

use crate::tiler::tests::fake_platform::Call;
use crate::{Action, Direction, Rect};

use super::{
    FakePlatform, Manager, REFRESH, animated, frame_by_frame, placements, settle, swap, two,
};

/// Two windows that swap, where window 1's application takes three frames
/// to get to each: it falls behind, and the swap ends with its placement
/// queued behind its first frame, while the desktop still has it in the
/// slot it left. Gives the two slots too, left and right.
pub(super) fn slow_swap() -> (FakePlatform, Manager, [Rect; 2]) {
    let (platform, mut manager) = two(&animated());
    let slots = [1, 2].map(|id| platform.window(id).rect);
    platform.edit(1, |w| w.frame_delay = 3);
    swap(&mut manager);
    frame_by_frame(&platform, &mut manager);
    platform.take_calls();
    (platform, manager, slots)
}

#[test]
fn a_swap_undone_before_a_slow_window_got_to_it_puts_both_back() {
    // Arrange
    let (platform, mut manager, [left, right]) = slow_swap();

    // Act -- the swap is undone, which sends window 1 back to where the
    // desktop still has it; then its application gets to all of it.
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();
    settle(&platform, &mut manager);
    platform.catch_up(1);

    // Assert -- each in the slot it started in.
    assert_eq!(platform.window(1).rect, left);
    assert_eq!(platform.window(2).rect, right);
}

#[test]
fn a_window_stopped_on_its_way_is_placed_behind_its_frame_as_it_comes_back() {
    // Arrange -- one frame of a swap is on its way to window 1 as the swap
    // is undone, and a workspace switch away and back by cloaking stops
    // window 1 where the desktop has it, in its slot.
    let (platform, mut manager) = two(&animated());
    let left = platform.window(1).rect;
    platform.edit(1, |w| w.frame_delay = 3);
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();

    // Act -- then its application gets to what is queued for it.
    manager.handle_action(&Action::GoToWorkspace(2));
    manager.handle_action(&Action::GoToWorkspace(1));
    manager.flush_pending();
    settle(&platform, &mut manager);
    platform.catch_up(1);

    // Assert -- in its slot, not where the frame took it.
    assert_eq!(platform.window(1).rect, left);
}

#[test]
fn the_pointer_follows_focus_to_where_a_window_is_going() {
    // Arrange
    let (platform, mut manager, [_, right]) = slow_swap();

    // Act -- focus goes to window 2 and back, the pointer following.
    manager.handle_action(&Action::Focus(Direction::Left));
    manager.handle_action(&Action::Focus(Direction::Right));
    manager.flush_pending();

    // Assert -- to the slot window 1 is on its way to.
    let calls = platform.take_calls();
    let (x, y) = (right.center_x(), right.center_y());
    assert!(calls.contains(&Call::MoveCursor(x, y)), "{calls:?}");
}

#[test]
fn a_window_in_place_and_last_sent_there_is_not_placed_again_while_that_is_recent() {
    // Arrange -- a swap is over, and both windows got to their placements.
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    settle(&platform, &mut manager);
    platform.take_calls();

    // Act -- the layout is laid out again right after.
    manager.handle_action(&Action::Retile);

    // Assert -- nothing is placed again.
    let calls = platform.take_calls();
    assert!(placements(&calls).is_empty(), "{calls:?}");
}

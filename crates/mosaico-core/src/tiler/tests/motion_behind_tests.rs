//! Windows whose applications handle their frames late, or fall behind.

use std::time::Duration;

use crate::WindowEvent;
use crate::id::wid;
use crate::tiler::Platform;
use crate::tiler::tests::fake_platform::Call;

use super::{
    FakePlatform, REFRESH, animated, frame_by_frame, placed_in, placements, start, step, swap, two,
    wait_for_landing,
};

#[test]
fn a_window_slow_to_handle_its_frames_is_placed_at_the_end_of_its_motion() {
    // Arrange -- window 1's application takes two frames to get to each.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    platform.edit(1, |w| w.frame_delay = 2);
    let began = platform.now();
    swap(&mut manager);

    // Act -- then its application gets to what is still queued for it.
    let steps = frame_by_frame(&platform, &mut manager);
    let took = platform.now() - began;
    platform.catch_up(1);

    // Assert -- placed with window 2 at the last step, once their motions
    // are over, behind the frames still on their way to it, and in its
    // slot once they land.
    let duration = animated().animations.spec().duration;
    assert_eq!(placed_in(&steps, 1), Some(steps.len()));
    assert_eq!(placed_in(&steps, 2), Some(steps.len()));
    assert!(took >= duration, "placed after {took:?}");
    assert_eq!(platform.window(1).rect, right);
    assert_eq!(platform.window(2).rect, left);
}

#[test]
fn a_window_that_falls_behind_ends_every_motion_of_its_step_at_once() {
    // Arrange -- window 2's application stops responding once it got to
    // the first frame of their swap.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    swap(&mut manager);
    assert!(step(&platform, &mut manager));
    platform.edit(2, |w| w.hung = true);

    // Act -- frame steps as the daemon takes them; then window 2's
    // application responds again, and gets to what was posted to it.
    let steps = frame_by_frame(&platform, &mut manager);
    let queued = platform.window(2).queued.len();
    platform.edit(2, |w| w.hung = false);
    platform.catch_up(2);

    // Assert -- window 2 is found still to handle its frame, and is not
    // seen to move, for 50 ms since it handled its first by the fourth
    // step after, so both are placed then, long before their motions would
    // end. Window 2's placement waits for nothing: it is posted behind its
    // frame, and lands once its application gets to it.
    assert_eq!(placed_in(&steps, 1), Some(4));
    assert_eq!(placed_in(&steps, 2), Some(4));
    assert_eq!(steps.len(), 4);
    assert_eq!(queued, 1);
    assert_eq!(platform.window(1).rect, right);
    assert_eq!(platform.window(2).rect, left);
}

#[test]
fn a_window_that_stopped_responding_neither_keeps_frames_coming_nor_stops_the_pointer() {
    // Arrange -- window 2 falls behind, which ends the swap.
    let (platform, mut manager) = two(&animated());
    platform.edit(2, |w| w.hung = true);
    swap(&mut manager);
    for _ in 0..4 {
        platform.advance(REFRESH);
        manager.frame();
    }
    platform.take_calls();

    // Act
    manager.handle_event(&WindowEvent::MouseHover { window: wid(2) });
    manager.flush_pending();
    let calls = platform.take_calls();

    // Assert -- nothing is left to ask about it once it stops being waited
    // for to land.
    assert!(!manager.wants_frames());
    wait_for_landing(&platform, &mut manager);
    assert_eq!(manager.frame_due(Some(platform.now())), None);
    assert!(calls.contains(&Call::Focus(wid(2))), "{calls:?}");
}

#[test]
fn a_window_that_falls_behind_ends_only_the_motions_of_its_own_step() {
    // Arrange -- windows 1 and 2 swap on one monitor. Two frames later
    // windows 3 and 4 swap on the other, where window 3's application has
    // stopped responding.
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);
    let mut manager = start(&platform, None, &animated());
    let began = platform.now();
    swap(&mut manager);
    for _ in 0..2 {
        assert!(step(&platform, &mut manager));
    }
    platform.edit(3, |w| w.hung = true);
    manager.handle_event(&WindowEvent::Focused { window: wid(3) });
    swap(&mut manager);
    platform.take_calls();

    // Act -- frame steps until window 1 is placed.
    let mut placed_after = None;
    while placed_after.is_none() {
        assert!(step(&platform, &mut manager), "window 1 is placed");
        let placed = placements(&platform.take_calls());
        if placed.iter().any(|&(window, _)| window == 1) {
            placed_after = Some(platform.now() - began);
        }
    }

    // Assert -- window 3 falling behind ends its own step, not the one
    // before: window 1 follows its curve to the end.
    let duration = animated().animations.spec().duration;
    assert!(
        placed_after >= Some(duration),
        "placed after {placed_after:?}"
    );
}

#[test]
fn a_window_a_frame_step_late_does_not_fall_behind_for_a_late_first_frame() {
    // Arrange -- window 1's application gets to each frame one frame step
    // late, and the first frame step comes 60 ms after the swap, as after a
    // slow apply or a flush ahead of it in the same wake.
    let (platform, mut manager) = two(&animated());
    platform.edit(1, |w| w.frame_delay = 1);
    let began = platform.now();
    swap(&mut manager);
    platform.advance(Duration::from_millis(60));
    manager.frame();

    // Act
    let steps = frame_by_frame(&platform, &mut manager);

    // Assert -- placed at the end of their motions, not on the next step.
    let duration = animated().animations.spec().duration;
    assert!(platform.now() - began >= duration);
    assert_eq!(placed_in(&steps, 1), Some(steps.len()));
}

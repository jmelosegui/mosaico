//! Placements of moving windows as they are applied: how far each window
//! travels is measured from where its motion has it, without reading the
//! desktop mid-flight, and a new motion starts from where the window was
//! last sent while that may still be on its way.

use std::time::Duration;

use crate::id::wid;
use crate::motion::Easing;
use crate::tiler::tests::fake_desktop::Queued;
use crate::{Action, Direction};

use super::{REFRESH, animated, frame_by_frame, placements, posted, swap, two};

#[test]
fn a_window_its_motion_has_almost_at_its_target_is_placed_there_at_once() {
    // Arrange -- a slow, even swap, where window 1's application takes a
    // frame step to get to each frame: three frames, the second of which
    // finds window 1 still to handle the first, then none until the motions
    // are 2 ms from their end. That has each window about 2 pixels from its
    // target, while the desktop still has it where its frames put it.
    let mut config = animated();
    config.animations.duration = 1000;
    config.animations.easing = Easing::Linear;
    let (platform, mut manager) = two(&config);
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    platform.edit(1, |w| w.frame_delay = 1);
    swap(&mut manager);
    for _ in 0..3 {
        platform.advance(REFRESH);
        manager.frame();
    }
    platform.advance(Duration::from_millis(998) - REFRESH * 3);
    platform.take_calls();

    // Act -- both are laid out again where they are going.
    manager.handle_action(&Action::Retile);

    // Assert -- placed in full at once, as they hardly move.
    assert_eq!(placements(&platform.take_calls()), [(2, left), (1, right)]);
    assert!(!manager.wants_frames());
}

#[test]
fn a_new_motion_starts_from_where_the_window_was_last_sent() {
    // Arrange -- window 1's application is busy and falls behind in a
    // swap, which ends with its placement queued behind its first frame;
    // then it gets to that frame only.
    let (platform, mut manager) = two(&animated());
    platform.edit(1, |w| w.frame_delay = 40);
    swap(&mut manager);
    frame_by_frame(&platform, &mut manager);
    let Some(&(first @ Queued::Frame(frame), _)) = platform.window(1).queued.first() else {
        panic!("a frame on its way");
    };
    platform.catch_up_through(1, first);

    // Act -- the swap is undone.
    platform.take_calls();
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();
    platform.advance(REFRESH);
    manager.frame();

    // Assert -- its next frame is on its way back from its slot, where it
    // was last sent, not from where its first frame left it.
    let calls = platform.take_calls();
    let frames = posted(&calls).concat();
    let back = frames.iter().find(|f| f.window == wid(1)).expect("a frame");
    assert!(back.rect.x > frame.x, "{back:?} after {frame:?}");
}

//! How the fake platform keeps a window's placements and show state
//! changes in order with the frames posted to it: queued behind them
//! while something may be queued, as a platform posts them, and made at
//! once otherwise.

use crate::config::HidingBehaviour;
use crate::frames::ASYNC_GRACE;
use crate::tiler::tests::fake_desktop::Queued;

use super::*;

#[path = "transition_fake_posted_tests.rs"]
mod posted_tests;
#[path = "transition_fake_settle_tests.rs"]
mod settle_tests;

/// What is queued for window 1, oldest first.
fn queued(platform: &FakePlatform) -> Vec<Queued> {
    let window = platform.window(1);
    window.queued.into_iter().map(|(q, _)| q).collect()
}

/// A platform whose window 1 has a frame at 120 queued behind its slow
/// application.
fn frame_queued() -> FakePlatform {
    let platform = FakePlatform::new(1, &[(1, 0)]);
    platform.edit(1, |w| w.frame_delay = 3);
    post(&platform, &[frame(120, true)]);
    platform
}

#[test]
fn a_placement_made_right_after_a_frame_lands_after_it() {
    // Arrange
    let platform = frame_queued();
    let target = Rect::new(400, 100, 640, 480);

    // Act -- the motion ends with its target.
    platform.place(wid(1), target).expect("placing works");
    let backlog = queued(&platform);
    platform.catch_up(1);

    // Assert
    let frame_at = frame(120, false).rect;
    assert_eq!(
        backlog,
        vec![Queued::Frame(frame_at), Queued::Place(target)]
    );
    assert_eq!(platform.window(1).rect, target);
}

#[test]
fn a_placement_with_nothing_posted_before_it_is_made_at_once() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let target = Rect::new(400, 100, 640, 480);

    // Act
    platform.place(wid(1), target).expect("placing works");

    // Assert
    assert_eq!(platform.window(1).rect, target);
    assert!(queued(&platform).is_empty());
}

#[test]
fn a_minimize_and_the_show_after_it_queue_behind_a_frame_in_order() {
    // Arrange
    let platform = frame_queued();

    // Act -- the workspace is left and come back to.
    let how = HidingBehaviour::Minimize;
    platform.hide(wid(1), how).expect("hiding works");
    platform.show(wid(1), how).expect("showing works");
    let backlog = queued(&platform);
    platform.catch_up(1);

    // Assert -- shown in the end, where the frame took it.
    let frame_at = frame(120, false).rect;
    let expected = [Queued::Frame(frame_at), Queued::Minimize, Queued::Show];
    assert_eq!(backlog, expected.to_vec());
    assert_eq!(platform.window(1).rect, frame_at);
    assert!(!platform.window(1).minimized);
}

#[test]
fn a_hide_behind_a_frame_is_queued_too_and_cloaking_never_is() {
    // Arrange
    let platform = frame_queued();

    // Act
    platform
        .hide(wid(1), HidingBehaviour::Cloak)
        .expect("cloaking works");
    platform
        .hide(wid(1), HidingBehaviour::Hide)
        .expect("hiding works");
    let (cloaked, visible) = (platform.window(1).cloaked, platform.window(1).visible);
    platform.catch_up(1);

    // Assert
    assert!(cloaked && visible);
    assert!(!platform.window(1).visible);
}

#[test]
fn what_is_queued_lands_in_order_as_the_calls_go_by() {
    // Arrange -- a frame, and a call later a minimize behind it, each
    // waiting three calls.
    let platform = frame_queued();
    post(&platform, &[frame(120, false)]);
    platform.minimize(wid(1)).expect("the window exists");

    // Act -- one call a frame, as the motion goes on.
    let mut seen = Vec::new();
    for _ in 0..4 {
        post(&platform, &[frame(120, false)]);
        let w = platform.window(1);
        seen.push((w.rect == frame(120, false).rect, w.minimized));
    }

    // Assert
    let expected = [(false, false), (false, false), (true, false), (true, true)];
    assert_eq!(seen, expected);
}

#[test]
fn a_change_long_after_the_last_post_is_made_at_once() {
    // Arrange
    let platform = frame_queued();
    platform.catch_up(1);
    platform.advance(ASYNC_GRACE);

    // Act
    platform.minimize(wid(1)).expect("the window exists");

    // Assert
    assert!(platform.window(1).minimized);
    assert!(queued(&platform).is_empty());
}

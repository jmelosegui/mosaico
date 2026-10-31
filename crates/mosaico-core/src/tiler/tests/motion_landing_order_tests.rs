//! What decides whether the placement that ends a motion is waited for:
//! where the desktop has the window, and where the last frame posted to it
//! took it, to the pixel. A new motion of the window ends the wait for the
//! placement before, and a border dropped meanwhile is not drawn once the
//! window lands.

use std::time::Duration;

use crate::config::{RuleAction, WindowRule};
use crate::id::wid;
use crate::tiler::tests::fake_platform::Call;
use crate::{Action, Direction, WindowEvent};

use super::super::posted_tests::slow_swap;
use super::super::{animated, bordered_at, frame_by_frame, placed_in, swap, two};
use super::bordered;

#[test]
fn a_slow_window_whose_frames_end_about_its_slot_is_bordered_once_it_lands() {
    for duration in [170, 180] {
        // Arrange -- window 1's application takes two frames to get to each,
        // so as its motion ends it is well short of its slot, with its last
        // frame, at its slot or a pixel short of it, still queued.
        let mut config = animated();
        config.animations.duration = duration;
        let (platform, mut manager) = two(&config);
        let right = platform.window(2).rect;
        platform.edit(1, |w| w.frame_delay = 2);
        swap(&mut manager);

        // Act -- then it gets to what is queued, and reports where that
        // leaves it.
        let ended = frame_by_frame(&platform, &mut manager).concat();
        platform.catch_up(1);
        manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });

        // Assert -- its border is not drawn ahead of it, but once it lands.
        assert!(!bordered(&ended, 1), "{duration}: {ended:?}");
        let calls = platform.take_calls();
        assert!(bordered_at(&calls, 1, right), "{duration}: {calls:?}");
    }
}

#[test]
fn a_window_its_last_frame_left_a_pixel_short_is_repainted_again_once_placed() {
    // Arrange -- at 160 ms, window 1's last frame leaves it a pixel short of
    // its slot.
    let mut config = animated();
    config.animations.duration = 160;
    let (platform, mut manager) = two(&config);
    let right = platform.window(2).rect;
    swap(&mut manager);

    // Act -- its placement moves it that pixel, which it reports.
    let steps = frame_by_frame(&platform, &mut manager);
    let landed = platform.take_calls();

    // Assert -- repainted again, and bordered, once it reports being there.
    let ended = steps.last().expect("frame steps");
    assert!(!bordered(ended, 1), "{ended:?}");
    assert!(landed.contains(&Call::Repaint(wid(1))), "{landed:?}");
    assert!(bordered_at(&landed, 1, right), "{landed:?}");
}

#[test]
fn a_window_that_moves_on_is_not_repainted_on_its_way_for_the_placement_before() {
    // Arrange -- window 1, slow, gets to its placement without reporting
    // it, and 400 ms later the swap is undone.
    let (platform, mut manager, _) = slow_swap();
    platform.catch_up(1);
    platform.advance(Duration::from_millis(400));

    // Act
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();
    let steps = frame_by_frame(&platform, &mut manager);

    // Assert -- not repainted while frames still go to it.
    let placed = placed_in(&steps, 1).expect("window 1 is placed");
    for calls in &steps[..placed - 1] {
        assert!(!calls.contains(&Call::Repaint(wid(1))), "{calls:?}");
    }
}

#[test]
fn a_window_let_go_of_while_its_placement_is_on_its_way_gets_no_border_as_it_lands() {
    // Arrange -- window 1, slow, has its placement on its way, with the
    // border its motion kept; then the rules let go of it, which drops its
    // border.
    let (platform, mut manager, _) = slow_swap();
    platform.edit(1, |w| {
        w.facts.iter_mut().for_each(|f| f.title = "let go".into())
    });
    manager.reload_rules(vec![WindowRule {
        match_class: None,
        match_title: Some("let go".into()),
        match_path: None,
        action: RuleAction::Ignore,
    }]);
    platform.take_calls();

    // Act -- then it gets to its placement, and reports it.
    platform.catch_up(1);
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(1) });

    // Assert
    let calls = platform.take_calls();
    assert!(!bordered(&calls, 1), "{calls:?}");
}

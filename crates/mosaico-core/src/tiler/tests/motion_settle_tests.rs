//! Windows given frames once transitions are off: the platform is asked to
//! forget them as soon as it can, so changes to them are made at once
//! again, and is not asked once it has.

use crate::config::Config;
use crate::frames::{ASYNC_GRACE, FrameControl};
use crate::id::wid;
use crate::tiler::tests::fake_platform::Call;
use crate::{Action, Direction};

use super::{FakePlatform, Manager, animated, settle, swap, two};

#[path = "motion_settle_step_tests.rs"]
mod step_tests;

/// Two windows that swapped with transitions on, both still posted to.
fn swapped() -> (FakePlatform, Manager) {
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    settle(&platform, &mut manager);
    (platform, manager)
}

/// How many times `calls` ask the platform to forget the windows it can.
fn settles(calls: &[Call]) -> usize {
    calls
        .iter()
        .filter(|&c| *c == Call::SettleResponding)
        .count()
}

#[test]
fn turning_transitions_off_has_the_platform_forget_the_windows_it_can_until_it_has() {
    // Arrange
    let (platform, mut manager) = swapped();
    let on = platform.take_calls();

    // Act -- a reload turns transitions off; seconds later, the layout is
    // laid out again twice.
    manager.reload_config(&Config::default());
    let reloaded = platform.take_calls();
    platform.advance(ASYNC_GRACE * 2);
    manager.handle_action(&Action::Retile);
    let past_the_grace = platform.take_calls();
    manager.handle_action(&Action::Retile);

    // Assert -- asked as they turn off, and on each step until both are
    // forgotten, but not while transitions are on.
    assert_eq!(settles(&on), 0, "{on:?}");
    assert_eq!(settles(&reloaded), 1, "{reloaded:?}");
    assert_eq!(settles(&past_the_grace), 1, "{past_the_grace:?}");
    assert_eq!(settles(&platform.take_calls()), 0);
    assert!(!platform.posts_behind(wid(1)) && !platform.posts_behind(wid(2)));
}

#[test]
fn a_window_forgotten_once_transitions_are_off_is_placed_at_once_while_it_hangs() {
    // Arrange -- transitions are off, and the windows were forgotten past
    // the grace; then window 1's application stops responding.
    let (platform, mut manager) = swapped();
    let right = platform.window(1).rect;
    let left = platform.window(2).rect;
    manager.reload_config(&Config::default());
    platform.advance(ASYNC_GRACE * 2);
    manager.handle_action(&Action::Retile);
    platform.edit(1, |w| w.hung = true);

    // Act -- the swap is undone.
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();

    // Assert -- placed at once, not queued behind what it does not get to.
    let one = platform.window(1);
    assert_eq!(one.rect, left, "{one:?}");
    assert!(one.queued.is_empty(), "{one:?}");
    assert_eq!(platform.window(2).rect, right);
}

#[test]
fn the_system_turning_animations_off_has_the_platform_forget_the_windows_it_can() {
    // Arrange -- the config follows the system's setting.
    let (platform, mut manager) = swapped();
    platform.take_calls();

    // Act -- the user turns animations off, and the layout is laid out
    // again.
    platform.allow_animations(false);
    manager.handle_action(&Action::Retile);

    // Assert
    assert_eq!(settles(&platform.take_calls()), 1);
}

#[test]
fn a_platform_that_turns_out_to_have_no_frames_is_asked_to_forget_the_windows_it_can() {
    // Arrange
    let (platform, mut manager) = swapped();
    platform.take_calls();

    // Act -- the daemon cannot have display frames.
    manager.turn_off_transitions();

    // Assert
    assert_eq!(settles(&platform.take_calls()), 1);
}

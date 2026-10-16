//! Transitions in the tiling manager, on the fake platform and its clock.

use std::time::Duration;

use crate::config::Config;
use crate::tiler::tests::fake_platform::{Call, FakePlatform};
use crate::tiler::tests::manager_support::start;
use crate::tiler::{Platform, TilingManager};
use crate::{Action, Direction, WindowEvent};

#[path = "motion_apply_tests.rs"]
mod apply_tests;
#[path = "motion_behind_tests.rs"]
mod behind_tests;
#[path = "motion_border_tests.rs"]
mod border_tests;
#[path = "motion_cursor_tests.rs"]
mod cursor_tests;
#[path = "motion_display_tests.rs"]
mod display_tests;
#[path = "motion_end_tests.rs"]
mod end_tests;
#[path = "motion_frame_tests.rs"]
mod frame_tests;
#[path = "motion_hidden_tests.rs"]
mod hidden_tests;
#[path = "motion_hiding_tests.rs"]
mod hiding_tests;
#[path = "motion_hover_tests.rs"]
mod hover_tests;
#[path = "motion_kept_border_tests.rs"]
mod kept_border_tests;
#[path = "motion_land_tests.rs"]
mod land_tests;
#[path = "motion_landed_tests.rs"]
mod landed_tests;
#[path = "motion_landing_tests.rs"]
mod landing_tests;
#[path = "motion_late_tests.rs"]
mod late_tests;
#[path = "motion_off_tests.rs"]
mod off_tests;
#[path = "motion_posted_tests.rs"]
mod posted_tests;
#[path = "motion_reload_tests.rs"]
mod reload_tests;
#[path = "motion_settle_tests.rs"]
mod settle_tests;
#[path = "motion_shown_tests.rs"]
mod shown_tests;
#[path = "motion_step_tests.rs"]
mod step_tests;
#[path = "motion_stop_tests.rs"]
mod stop_tests;
#[path = "motion_support.rs"]
mod support;
#[path = "motion_travel_tests.rs"]
mod travel_tests;
#[path = "motion_way_tests.rs"]
mod way_tests;

use support::{bordered_at, placed_in, placements, posted};

type Manager = TilingManager<FakePlatform>;

/// About one display refresh at 60 Hz.
const REFRESH: Duration = Duration::from_millis(16);

/// A config with transitions on.
fn animated() -> Config {
    let mut config = Config::default();
    config.animations.enabled = true;
    config
}

/// Two windows side by side on one monitor, managed with `config`, with
/// what starting took forgotten.
fn two(config: &Config) -> (FakePlatform, Manager) {
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let manager = start(&platform, None, config);
    platform.take_calls();
    (platform, manager)
}

/// Swaps the focused window with the one on its right, and lets input go
/// quiet.
fn swap(manager: &mut Manager) {
    manager.handle_action(&Action::Move(Direction::Right));
    manager.flush_pending();
}

/// Takes frame steps as the daemon does until no window moves, and
/// returns how many it took, leaving what they asked of the platform to
/// be taken. Then [`land_prompt`]. Fails the test after 64 of them.
fn settle(platform: &FakePlatform, manager: &mut Manager) -> usize {
    for steps in 0..64 {
        if !step(platform, manager) {
            land_prompt(platform, manager);
            return steps;
        }
    }
    panic!("windows still moving after 64 frame steps");
}

/// Takes frame steps as the daemon does until no window moves, and
/// returns what each step asked of the platform. Then [`land_prompt`].
/// Fails the test after 64 of them.
fn frame_by_frame(platform: &FakePlatform, manager: &mut Manager) -> Vec<Vec<Call>> {
    let mut steps = Vec::new();
    for _ in 0..64 {
        if !step(platform, manager) {
            land_prompt(platform, manager);
            return steps;
        }
        steps.push(platform.take_calls());
    }
    panic!("windows still moving after 64 frame steps");
}

/// The applications that respond promptly get to what is still queued for
/// them, and the platform reports where that leaves each window that moved.
fn land_prompt(platform: &FakePlatform, manager: &mut Manager) {
    for window in platform.catch_up_prompt() {
        manager.handle_event(&WindowEvent::LocationChanged { window });
    }
}

/// Takes the next frame step as the daemon does while a window moves: on
/// the next display refresh. Says whether one was taken.
fn step(platform: &FakePlatform, manager: &mut Manager) -> bool {
    if !manager.wants_frames() {
        return false;
    }
    platform.advance(REFRESH);
    manager.frame();
    true
}

/// Lets time pass to the frame step the daemon takes once it stops waiting
/// for a window to land, and takes it. Fails the test if none is due.
fn wait_for_landing(platform: &FakePlatform, manager: &mut Manager) {
    let due = manager
        .frame_due(Some(platform.now()))
        .expect("a frame step due");
    platform.advance(due.saturating_duration_since(platform.now()));
    manager.frame();
}

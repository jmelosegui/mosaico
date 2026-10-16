//! Transitions in the tiling manager, on the fake platform and its clock.

use std::time::Duration;

use crate::config::Config;
use crate::tiler::TilingManager;
use crate::tiler::tests::fake_platform::{Call, FakePlatform};
use crate::tiler::tests::manager_support::start;
use crate::{Action, Direction, Rect};

#[path = "motion_apply_tests.rs"]
mod apply_tests;
#[path = "motion_reload_tests.rs"]
mod reload_tests;

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

/// Takes frames a display refresh apart until no window moves, and
/// returns how many it took. Fails the test after a second of them.
fn settle(platform: &FakePlatform, manager: &mut Manager) -> usize {
    for frames in 0..64 {
        if !manager.wants_frames() {
            return frames;
        }
        platform.advance(REFRESH);
        manager.frame();
    }
    panic!("windows still moving after a second of frames");
}

/// The placements among `calls`, in order.
fn placements(calls: &[Call]) -> Vec<(u64, Rect)> {
    let placed = |call: &Call| match *call {
        Call::Place(window, rect) => Some((window.raw(), rect)),
        _ => None,
    };
    calls.iter().filter_map(placed).collect()
}

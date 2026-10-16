//! Running a metrics scenario and catching what it asks of the platform.

use std::time::Duration;

use crate::Direction;
use crate::config::Config;

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::start;
use super::*;

pub(super) type Manager = TilingManager<FakePlatform>;

/// What a scenario asked of the platform.
pub(super) struct Measured {
    /// Every change asked for.
    pub(super) calls: Vec<Call>,
    /// How many borders were drawn from scratch rather than moved.
    pub(super) renders: usize,
}

/// Windows `1..=count` on monitor 0.
pub(super) fn windows(count: u64) -> Vec<(u64, usize)> {
    (1..=count).map(|id| (id, 0)).collect()
}

/// Four windows on one monitor, managed.
pub(super) fn four() -> (FakePlatform, Manager) {
    let platform = FakePlatform::new(1, &windows(4));
    let manager = start(&platform, None, &Config::default());
    (platform, manager)
}

/// Nine presses alternating right and left, faster than input goes
/// quiet, so they end one slot from where they started.
pub(super) fn burst(press: fn(Direction) -> Action) -> Vec<Action> {
    (0..9)
        .map(|i| {
            press(if i % 2 == 0 {
                Direction::Right
            } else {
                Direction::Left
            })
        })
        .collect()
}

/// Everything asked of `platform` since it was last asked, forgetting it.
pub(super) fn taken(platform: &FakePlatform) -> Measured {
    Measured {
        calls: platform.take_calls(),
        renders: platform.take_renders().len(),
    }
}

/// Forgets what arranging took, runs `change` and lets input go quiet.
pub(super) fn on(arranged: (FakePlatform, Manager), change: impl FnOnce(&mut Manager)) -> Measured {
    let (platform, mut manager) = arranged;
    taken(&platform);
    change(&mut manager);
    manager.flush_pending();
    taken(&platform)
}

/// Runs `actions` on four windows.
pub(super) fn act(actions: &[Action]) -> Measured {
    on(four(), |manager| {
        for action in actions {
            manager.handle_action(action);
        }
    })
}

/// Runs `actions` on four windows with transitions on, then takes frames
/// as the daemon does until no window moves, a display refresh apart.
/// Then the applications get to what is queued for them, and the platform
/// reports the windows that moved, as Windows does. A window still to land
/// after that is waited for, until the frame step that ends the wait by
/// time.
pub(super) fn animate(actions: &[Action]) -> Measured {
    let platform = FakePlatform::new(1, &windows(4));
    let mut config = Config::default();
    config.animations.enabled = true;
    let manager = start(&platform, None, &config);
    on((platform.clone(), manager), |manager| {
        for action in actions {
            manager.handle_action(action);
        }
        manager.flush_pending();
        for _ in 0..64 {
            if manager.wants_frames() {
                platform.advance(Duration::from_millis(16));
            } else {
                for window in platform.catch_up_prompt() {
                    manager.handle_event(&WindowEvent::LocationChanged { window });
                }
                let Some(due) = manager.frame_due(Some(platform.now())) else {
                    return;
                };
                platform.advance(due.saturating_duration_since(platform.now()));
            }
            manager.frame();
        }
        panic!("windows still moving after 64 frame steps");
    })
}

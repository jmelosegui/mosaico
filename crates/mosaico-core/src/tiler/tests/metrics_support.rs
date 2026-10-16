//! Running a metrics scenario and catching what it asks of the platform.

use crate::Direction;
use crate::config::Config;

use super::fake_platform::{Call, FakePlatform};
use super::manager_support::{animated, settle, start, still};
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

/// Four windows on one monitor, managed with transitions off.
pub(super) fn four() -> (FakePlatform, Manager) {
    four_with(&still())
}

/// Four windows on one monitor, managed with `config`.
pub(super) fn four_with(config: &Config) -> (FakePlatform, Manager) {
    let platform = FakePlatform::new(1, &windows(4));
    let manager = start(&platform, None, config);
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

/// Runs `actions` on four windows with transitions on, as [`animated`]
/// Runs `actions` on four windows with transitions on, as [`on_settled`]
/// does.
pub(super) fn animate(actions: &[Action]) -> Measured {
    on_settled(four_with(&animated()), |manager| {
        for action in actions {
            manager.handle_action(action);
        }
    })
}

/// Forgets what arranging took, runs `change` and lets input go quiet,
/// then lets the windows settle, as the daemon's frames and the platform's
/// reports do.
pub(super) fn on_settled(
    arranged: (FakePlatform, Manager),
    change: impl FnOnce(&mut Manager),
) -> Measured {
    let platform = arranged.0.clone();
    on(arranged, |manager| {
        change(manager);
        manager.flush_pending();
        settle(&platform, manager);
    })
}

//! The display scenarios: a monitor disconnected, and connected again.

use super::super::displays;
use super::super::fake_platform::FakePlatform;
use super::super::manager_support::{start, still};
use super::super::metrics_support::{Manager, Measured, on};

/// Two windows on each of two monitors.
fn two_monitors() -> (FakePlatform, Manager) {
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);
    let manager = start(&platform, None, &still());
    (platform, manager)
}

pub(super) fn disconnect() -> Measured {
    on(two_monitors(), |m| {
        m.handle_display_change(displays(1), 0, &[]);
    })
}

pub(super) fn reconnect() -> Measured {
    let (platform, mut manager) = two_monitors();
    manager.handle_display_change(displays(1), 0, &[]);
    manager.flush_pending();
    on((platform, manager), |m| {
        m.handle_display_change(displays(2), 0, &[]);
    })
}

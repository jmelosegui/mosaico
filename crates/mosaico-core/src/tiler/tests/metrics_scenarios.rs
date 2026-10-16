//! What the tiling manager asks the platform to do in common situations,
//! counted for the metrics baseline.
//!
//! Each scenario arranges a desktop, forgets what that took, does one
//! thing and lets input go quiet, so what is counted is what reaches the
//! platform for that one thing after coalescing.

use crate::Direction;
use crate::config::Config;

use super::fake_platform::FakePlatform;
use super::manager_support::start;
use super::metrics_support::{Manager, Measured, act, animate, burst, four, on, taken, windows};
use super::*;

/// A scenario: its name, and how to run it.
pub(super) type Scenario = (&'static str, fn() -> Measured);

/// Every scenario, in the order the baseline lists them.
pub(super) const SCENARIOS: &[Scenario] = &[
    ("startup_with_4", startup),
    ("open_next_to_1", || open_next_to(1)),
    ("open_next_to_3", || open_next_to(3)),
    ("open_next_to_8", || open_next_to(8)),
    ("close_one_of_4", close_one),
    ("focus_once", || act(&[Action::Focus(Direction::Right)])),
    ("focus_burst_of_9", || act(&burst(Action::Focus))),
    ("move_once", || act(&[Action::Move(Direction::Right)])),
    ("move_once_animated", || {
        animate(&[Action::Move(Direction::Right)])
    }),
    ("move_burst_of_9", || act(&burst(Action::Move))),
    ("retile", || act(&[Action::Retile])),
    ("cycle_layout", || act(&[Action::CycleLayout])),
    ("monocle_on_and_off", || {
        act(&[Action::ToggleMonocle, Action::ToggleMonocle])
    }),
    ("workspace_there_and_back", || {
        act(&[Action::GoToWorkspace(2), Action::GoToWorkspace(1)])
    }),
    ("send_to_workspace", || act(&[Action::SendToWorkspace(2)])),
    ("config_reload", config_reload),
    ("bar_reserved", || {
        on(four(), |m| m.adjust_work_areas_for_bar(30, &[]))
    }),
    ("monitor_disconnect", disconnect),
    ("monitor_reconnect", reconnect),
];

/// Starting on four open windows, counted from the start.
fn startup() -> Measured {
    let platform = FakePlatform::new(1, &windows(4));
    let mut manager = start(&platform, None, &Config::default());
    manager.flush_pending();
    taken(&platform)
}

fn open_next_to(count: u64) -> Measured {
    let mut open = windows(count);
    open.push((99, 0));
    let platform = FakePlatform::new(1, &open);
    platform.edit(99, |w| w.visible = false);
    let manager = start(&platform, None, &Config::default());
    platform.edit(99, |w| w.visible = true);
    on((platform, manager), |m| {
        m.handle_event(&WindowEvent::Created { window: wid(99) });
    })
}

fn close_one() -> Measured {
    let (platform, manager) = four();
    platform.close_window(2);
    on((platform, manager), |m| {
        m.handle_event(&WindowEvent::Destroyed { window: wid(2) });
    })
}

fn config_reload() -> Measured {
    let mut config = Config::default();
    config.layout.gap = 16;
    on(four(), |m| m.reload_config(&config))
}

/// Two windows on each of two monitors.
fn two_monitors() -> (FakePlatform, Manager) {
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);
    let manager = start(&platform, None, &Config::default());
    (platform, manager)
}

fn disconnect() -> Measured {
    on(two_monitors(), |m| {
        m.handle_display_change(displays(1), 0, &[]);
    })
}

fn reconnect() -> Measured {
    let (platform, mut manager) = two_monitors();
    manager.handle_display_change(displays(1), 0, &[]);
    manager.flush_pending();
    on((platform, manager), |m| {
        m.handle_display_change(displays(2), 0, &[]);
    })
}

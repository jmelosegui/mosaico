//! What the tiling manager asks the platform to do in common situations,
//! counted for the metrics baseline.
//!
//! Each scenario arranges a desktop, forgets what that took, does one
//! thing and lets input go quiet, so what is counted is what reaches the
//! platform for that one thing after coalescing. Those named `_animated`
//! run with transitions on, and count the frames until the windows have
//! settled too; the others run with transitions off.

use crate::Direction;
use crate::config::Config;

use super::fake_platform::FakePlatform;
use super::manager_support::{animated, settle, start, still};
use super::metrics_support::{
    Manager, Measured, act, animate, burst, four, four_with, on, on_settled, taken, windows,
};
use super::*;

/// A scenario: its name, and how to run it.
pub(super) type Scenario = (&'static str, fn() -> Measured);

/// Every scenario.
pub(super) const SCENARIOS: &[Scenario] = &[
    ("startup_with_4", startup),
    ("open_next_to_1", || on(before_opening(1, &still()), open)),
    ("open_next_to_3", || on(before_opening(3, &still()), open)),
    ("open_next_to_3_animated", || {
        on_settled(before_opening(3, &animated()), open)
    }),
    ("open_next_to_8", || on(before_opening(8, &still()), open)),
    ("close_one_of_4", || on(before_closing(&still()), close)),
    ("close_one_of_4_animated", || {
        on_settled(before_closing(&animated()), close)
    }),
    ("focus_once", || act(&[Action::Focus(Direction::Right)])),
    ("focus_burst_of_9", || act(&burst(Action::Focus))),
    ("move_once", || act(&[Action::Move(Direction::Right)])),
    ("move_once_animated", || {
        animate(&[Action::Move(Direction::Right)])
    }),
    ("move_burst_of_9", || act(&burst(Action::Move))),
    ("retile", || act(&[Action::Retile])),
    ("cycle_layout", || act(&[Action::CycleLayout])),
    ("cycle_layout_animated", || animate(&[Action::CycleLayout])),
    ("monocle_on_and_off", || {
        act(&[Action::ToggleMonocle, Action::ToggleMonocle])
    }),
    ("monocle_on_and_off_animated", monocle_animated),
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
    let mut manager = start(&platform, None, &still());
    manager.flush_pending();
    taken(&platform)
}

/// Windows `1..=count`, managed with `config`, and window 99, about to
/// open next to them.
fn before_opening(count: u64, config: &Config) -> (FakePlatform, Manager) {
    let mut open = windows(count);
    open.push((99, 0));
    let platform = FakePlatform::new(1, &open);
    platform.edit(99, |w| w.visible = false);
    let manager = start(&platform, None, config);
    platform.edit(99, |w| w.visible = true);
    (platform, manager)
}

/// Window 99 opens.
fn open(manager: &mut Manager) {
    manager.handle_event(&WindowEvent::Created { window: wid(99) });
}

/// Four windows, managed with `config`, and window 2 closed, not reported
/// yet.
fn before_closing(config: &Config) -> (FakePlatform, Manager) {
    let (platform, manager) = four_with(config);
    platform.close_window(2);
    (platform, manager)
}

/// Window 2 is reported closed.
fn close(manager: &mut Manager) {
    manager.handle_event(&WindowEvent::Destroyed { window: wid(2) });
}

/// Monocle on, and once the windows have settled, off again.
fn monocle_animated() -> Measured {
    let arranged = four_with(&animated());
    let platform = arranged.0.clone();
    on_settled(arranged, |manager| {
        manager.handle_action(&Action::ToggleMonocle);
        manager.flush_pending();
        settle(&platform, manager);
        manager.handle_action(&Action::ToggleMonocle);
    })
}

fn config_reload() -> Measured {
    let mut config = still();
    config.layout.gap = 16;
    on(four(), |m| m.reload_config(&config))
}

/// Two windows on each of two monitors.
fn two_monitors() -> (FakePlatform, Manager) {
    let platform = FakePlatform::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);
    let manager = start(&platform, None, &still());
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

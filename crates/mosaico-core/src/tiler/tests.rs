//! Tiler tests against a fake desktop, and tiling manager tests against
//! a fake platform.

use crate::action::MAX_WORKSPACES;
use crate::config::HidingBehaviour;
use crate::id::wid;
use crate::{Action, MonitorId, MonitorKey, Rect, WindowEvent, WindowId, Workspace};

use super::*;

use fake_desktop::{FakeDesktop, FakeWindow};
pub(super) use harness::Harness;

#[path = "tests/adopt_order_kept_tests.rs"]
mod adopt_order_kept_tests;
#[path = "tests/adopt_order_support.rs"]
mod adopt_order_support;
#[path = "tests/adopt_order_tests.rs"]
mod adopt_order_tests;
#[path = "tests/border_once_tests.rs"]
mod border_once_tests;
#[path = "tests/border_order_tests.rs"]
mod border_order_tests;
#[path = "tests/border_redraw_tests.rs"]
mod border_redraw_tests;
#[path = "tests/border_tests.rs"]
mod border_tests;
#[path = "tests/corner_tests.rs"]
mod corner_tests;
#[path = "tests/corner_tracking_tests.rs"]
mod corner_tracking_tests;
#[path = "tests/decoration_effects_tests.rs"]
mod decoration_effects_tests;
#[path = "tests/display_focus_tests.rs"]
mod display_focus_tests;
#[path = "tests/display_tests.rs"]
mod display_tests;
#[path = "tests/empty_focus_tests.rs"]
mod empty_focus_tests;
#[path = "tests/event_tests.rs"]
mod event_tests;
#[path = "tests/fake_desktop.rs"]
pub(super) mod fake_desktop;
#[path = "tests/fake_platform.rs"]
pub(super) mod fake_platform;
#[path = "tests/fallback_tests.rs"]
mod fallback_tests;
#[path = "tests/focus_tests.rs"]
mod focus_tests;
#[path = "tests/global_lineup_tests.rs"]
mod global_lineup_tests;
#[path = "tests/harness.rs"]
mod harness;
#[path = "tests/held_border_tests.rs"]
mod held_border_tests;
#[path = "tests/held_removal_tests.rs"]
mod held_removal_tests;
#[path = "tests/hidden_how_tests.rs"]
mod hidden_how_tests;
#[path = "tests/hiding_change_record_tests.rs"]
mod hiding_change_record_tests;
#[path = "tests/hiding_change_tests.rs"]
mod hiding_change_tests;
#[path = "tests/layout_tests.rs"]
mod layout_tests;
#[path = "tests/manager_config_tests.rs"]
mod manager_config_tests;
#[path = "tests/manager_daemon_tests.rs"]
mod manager_daemon_tests;
#[path = "tests/manager_focus_tests.rs"]
mod manager_focus_tests;
#[path = "tests/manager_record_tests.rs"]
mod manager_record_tests;
#[path = "tests/manager_support.rs"]
pub(super) mod manager_support;
#[path = "tests/manager_tests.rs"]
mod manager_tests;
#[path = "tests/metrics_scenarios.rs"]
mod metrics_scenarios;
#[path = "tests/metrics_support.rs"]
mod metrics_support;
#[path = "tests/metrics_tests.rs"]
mod metrics_tests;
#[path = "tests/min_size_layout_tests.rs"]
mod min_size_layout_tests;
#[path = "tests/mode_reload_settings_tests.rs"]
mod mode_reload_settings_tests;
#[path = "tests/mode_reload_tests.rs"]
mod mode_reload_tests;
#[path = "tests/monitor_return_tests.rs"]
mod monitor_return_tests;
#[path = "tests/monocle_display_tests.rs"]
mod monocle_display_tests;
#[path = "tests/monocle_join_tests.rs"]
mod monocle_join_tests;
#[path = "tests/monocle_out_of_view_tests.rs"]
mod monocle_out_of_view_tests;
#[path = "tests/monocle_return_tests.rs"]
mod monocle_return_tests;
#[path = "tests/monocle_stale_tests.rs"]
mod monocle_stale_tests;
#[path = "tests/move_tests.rs"]
mod move_tests;
#[path = "tests/pipeline_tests.rs"]
mod pipeline_tests;
#[path = "tests/posted_move_tests.rs"]
mod posted_move_tests;
#[path = "tests/pre_place_tests.rs"]
mod pre_place_tests;
#[path = "tests/rules_release_tests.rs"]
mod rules_release_tests;
#[path = "tests/rules_reload_tests.rs"]
mod rules_reload_tests;
#[path = "tests/session_tests.rs"]
mod session_tests;
#[path = "tests/stacking_tests.rs"]
mod stacking_tests;
#[path = "tests/window_effects_tests.rs"]
mod window_effects_tests;
#[path = "tests/window_recorder.rs"]
mod window_recorder;
#[path = "tests/work_area_tests.rs"]
mod work_area_tests;
#[path = "tests/workspace_tests.rs"]
mod workspace_tests;

/// Width of every fake monitor.
pub(super) const MON_W: i32 = 1920;
/// Height of every fake monitor.
pub(super) const MON_H: i32 = 1080;

/// `count` monitors side by side, the first one primary, with ids
/// `0..count` and distinct keys.
pub(super) fn displays(count: usize) -> Vec<Display> {
    (0..count)
        .map(|i| Display {
            id: MonitorId::from_raw(i as u64),
            key: key(&format!("mon{i}")),
            work_area: Rect::new(i as i32 * MON_W, 0, MON_W, MON_H),
            primary: i == 0,
        })
        .collect()
}

/// A monitor key with only a stable id.
pub(super) fn key(name: &str) -> MonitorKey {
    MonitorKey {
        stable_id: Some(name.into()),
        ..Default::default()
    }
}

/// Bare monitor state for tests of the free functions.
pub(super) fn make_monitor(id: u64) -> MonitorState {
    MonitorState {
        id: MonitorId::from_raw(id),
        key: MonitorKey::default(),
        os_work_area: Rect::new(0, 0, MON_W, MON_H),
        work_area: Rect::new(0, 0, MON_W, MON_H),
        workspaces: (0..MAX_WORKSPACES).map(|_| Workspace::new()).collect(),
        active_workspace: 0,
    }
}

pub(super) fn make_monitors(n: usize) -> Vec<MonitorState> {
    (0..n as u64).map(make_monitor).collect()
}

/// Runs the real display change rebuild with unidentified monitors, the
/// first of which is primary.
pub(super) fn simulate_display_change(
    old_monitors: &mut Vec<MonitorState>,
    new_infos: Vec<(u64, Rect)>,
) -> Vec<MonitorState> {
    let infos: Vec<Display> = new_infos
        .into_iter()
        .enumerate()
        .map(|(i, (id, work_area))| Display {
            id: MonitorId::from_raw(id),
            key: MonitorKey::default(),
            work_area,
            primary: i == 0,
        })
        .collect();
    let layouts = vec![LayoutKind::default(); MAX_WORKSPACES as usize];
    let mut memory = reconnect::ReconnectMemory::default();
    display_rebuild::rebuild_monitors(std::mem::take(old_monitors), &infos, &layouts, &mut memory)
        .monitors
}

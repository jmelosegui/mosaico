//! Keeps each monitor's tiling area in sync with the OS and the bar.

use mosaico_core::Rect;

use crate::monitor::MonitorInfo;

use super::{MonitorState, TilingManager};

impl TilingManager {
    /// Reserves the bar height at the top of each bar monitor, then retiles.
    ///
    /// The tiling area is always derived from the OS work area, so
    /// calling this repeatedly never stacks the offset. Only monitors
    /// whose index appears in `bar_monitors` are adjusted; an empty
    /// slice means all monitors.
    pub fn adjust_work_areas_for_bar(&mut self, bar_height: i32, bar_monitors: &[usize]) {
        apply_bar_reservation(&mut self.monitors, bar_height, bar_monitors);
        self.retile_all();
    }

    /// Re-reads the OS work areas, then re-applies the bar and retiles.
    ///
    /// Used when the taskbar, the bar config, or the user (via retile)
    /// asks for the tiling area to be recomputed from scratch.
    pub fn reset_and_adjust_work_areas(&mut self, bar_height: i32, bar_monitors: &[usize]) {
        if let Ok(os_monitors) = crate::monitor::enumerate_monitors() {
            sync_os_work_areas(&mut self.monitors, &os_monitors);
        }
        self.adjust_work_areas_for_bar(bar_height, bar_monitors);
    }
}

/// Copies fresh OS work areas into the matching monitor states.
///
/// Monitors are matched by ID. Reconnecting a display (e.g. through a
/// KVM switch) hands out new HMONITOR IDs, so when the monitor count is
/// unchanged an unmatched monitor falls back to the one at the same
/// left-to-right index and adopts its new ID.
pub(super) fn sync_os_work_areas(monitors: &mut [MonitorState], os_monitors: &[MonitorInfo]) {
    let same_count = monitors.len() == os_monitors.len();
    for (i, mon) in monitors.iter_mut().enumerate() {
        let os = os_monitors
            .iter()
            .find(|m| m.id == mon.id)
            .or_else(|| same_count.then(|| &os_monitors[i]));
        if let Some(os) = os {
            mon.id = os.id;
            mon.os_work_area = os.work_area;
        }
    }
}

/// Sets each monitor's tiling area to its OS work area minus the bar.
pub(super) fn apply_bar_reservation(
    monitors: &mut [MonitorState],
    bar_height: i32,
    bar_monitors: &[usize],
) {
    for (i, mon) in monitors.iter_mut().enumerate() {
        let has_bar = bar_monitors.is_empty() || bar_monitors.contains(&i);
        let reserved = if has_bar { bar_height } else { 0 };
        mon.work_area = below_bar(&mon.os_work_area, reserved);
    }
}

/// Returns `area` with `bar_height` pixels removed from the top.
pub(super) fn below_bar(area: &Rect, bar_height: i32) -> Rect {
    Rect::new(
        area.x,
        area.y + bar_height,
        area.width,
        area.height - bar_height,
    )
}

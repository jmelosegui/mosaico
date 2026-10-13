//! Keeps each monitor's tiling area in sync with the OS and the bar.

use crate::Rect;

use super::{BarReservation, Display, MonitorState, Plan, Tiler};

impl Tiler {
    /// Reserves the bar at the top of each bar monitor, then retiles.
    ///
    /// The tiling area is always derived from the OS work area, so doing
    /// this repeatedly never stacks the offset.
    pub(super) fn reserve_bar(&mut self, plan: &mut Plan, bar: &BarReservation) {
        apply_bar_reservation(&mut self.monitors, bar.height, &bar.monitors);
        self.retile_all(plan);
    }

    /// Takes fresh OS work areas, then reserves the bar and retiles.
    pub(super) fn reset_work_areas(
        &mut self,
        plan: &mut Plan,
        displays: &[Display],
        bar: &BarReservation,
    ) {
        sync_os_work_areas(&mut self.monitors, displays);
        self.reserve_bar(plan, bar);
    }
}

/// Copies fresh OS work areas into the matching monitor states.
///
/// Monitors are matched by key, then by session id. Reconnecting a
/// display (through a KVM switch, say) can hand out new session ids, so
/// when neither matches and the monitor count is unchanged, an
/// unmatched monitor falls back to the one at the same index and adopts
/// its new id.
pub(super) fn sync_os_work_areas(monitors: &mut [MonitorState], displays: &[Display]) {
    let same_count = monitors.len() == displays.len();
    for (i, mon) in monitors.iter_mut().enumerate() {
        let os = displays
            .iter()
            .find(|m| !mon.key.is_unknown() && m.key == mon.key)
            .or_else(|| displays.iter().find(|m| m.id == mon.id))
            .or_else(|| same_count.then(|| &displays[i]));
        if let Some(os) = os {
            mon.id = os.id;
            mon.os_work_area = os.work_area;
        }
    }
}

/// Sets each monitor's tiling area to its OS work area minus the bar.
/// An empty `bar_monitors` means every monitor has a bar.
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

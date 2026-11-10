//! Keeps each monitor's tiling area in sync with the OS and the bar.

use crate::Rect;

use super::{Display, MonitorState, Plan, Reach, Tiler};

impl Tiler {
    /// Keeps each monitor's tiling area clear of how far the bar reaches
    /// into it, then retiles.
    ///
    /// The tiling area is always derived from the OS work area, so doing
    /// this repeatedly never stacks the offset.
    pub(super) fn reserve_bar(&mut self, plan: &mut Plan, reach: &Reach) {
        apply_reach(&mut self.monitors, reach);
        self.retile_all(plan);
    }

    /// Reserves the bar like [`reserve_bar`](Self::reserve_bar), but only
    /// retiles when that moves a monitor's tiling area.
    pub(super) fn reserve_bar_if_moved(&mut self, plan: &mut Plan, reach: &Reach) {
        let before: Vec<Rect> = self.monitors.iter().map(|m| m.work_area).collect();
        apply_reach(&mut self.monitors, reach);
        if self.monitors.iter().map(|m| m.work_area).ne(before) {
            self.retile_all(plan);
        }
    }

    /// Takes fresh OS work areas, then reserves the bar and retiles.
    pub(super) fn reset_work_areas(
        &mut self,
        plan: &mut Plan,
        displays: &[Display],
        reach: &Reach,
    ) {
        sync_os_work_areas(&mut self.monitors, displays);
        self.reserve_bar(plan, reach);
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
            mon.sync(os);
        }
    }
}

/// Sets each monitor's tiling area to its OS work area kept clear of how
/// far the bar reaches down from the monitor's top edge. Whichever of the
/// two reaches further wins, so a bar the system already keeps windows
/// away from is never counted twice.
pub(super) fn apply_reach(monitors: &mut [MonitorState], reach: &Reach) {
    for mon in monitors {
        let line = mon.bounds.y + reach.top(&mon.stable_id);
        let below = (line - mon.os_work_area.y).max(0);
        mon.work_area = below_bar(&mon.os_work_area, below);
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

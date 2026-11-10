//! Keeps each monitor's tiling area in sync with the OS and the bar.

use crate::Rect;

use super::{Display, Edges, MonitorState, Plan, Reach, Side, Tiler};

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
    /// retiles the monitors whose tiling area that moves, so a bar that
    /// changes its space on one monitor leaves the others alone.
    pub(super) fn reserve_bar_if_moved(&mut self, plan: &mut Plan, reach: &Reach) {
        let before: Vec<Rect> = self.monitors.iter().map(|m| m.work_area).collect();
        apply_reach(&mut self.monitors, reach);
        for (i, was) in before.into_iter().enumerate() {
            if self.monitors[i].work_area != was {
                self.apply_layout_on(plan, i);
            }
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
/// far the bar reaches in from each of the monitor's edges. On each edge
/// whichever of the two reaches further wins, so a bar the system already
/// keeps windows away from is never counted twice.
pub(super) fn apply_reach(monitors: &mut [MonitorState], reach: &Reach) {
    for mon in monitors {
        let edges = reach.edges(&mon.stable_id);
        mon.work_area = kept_clear(&mon.os_work_area, &mon.bounds, edges);
    }
}

/// `os`, the work area of a monitor covering `bounds`, with each edge
/// moved in to where `edges` reach from the monitor's edge, when that is
/// further in. An edge `edges` does not reach from stays where the system
/// put it.
fn kept_clear(os: &Rect, bounds: &Rect, edges: Edges) -> Rect {
    let line = |side: Side, from: i32, sign: i32| match edges.get(side) {
        0 => None,
        pixels => Some(from + sign * pixels),
    };
    let top = line(Side::Top, bounds.y, 1).map_or(os.y, |y| y.max(os.y));
    let left = line(Side::Left, bounds.x, 1).map_or(os.x, |x| x.max(os.x));
    let os_bottom = os.y + os.height;
    let bottom =
        (line(Side::Bottom, bounds.y + bounds.height, -1)).map_or(os_bottom, |y| y.min(os_bottom));
    let os_right = os.x + os.width;
    let right =
        (line(Side::Right, bounds.x + bounds.width, -1)).map_or(os_right, |x| x.min(os_right));
    Rect::new(left, top, (right - left).max(0), (bottom - top).max(0))
}

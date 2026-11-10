//! How far the bar reaches into each monitor, and the monitors as a bar
//! needs to know them to say so.

use crate::{Rect, StableMonitorId};

use super::display_rebuild::{pair_monitors, stable_ids};
use super::{Display, Tiler};

impl Tiler {
    /// The connected monitors, left to right, as a bar needs to know them.
    pub fn places(&self) -> Vec<MonitorPlace> {
        (self.monitors.iter().enumerate())
            .map(|(index, mon)| MonitorPlace {
                id: mon.stable_id.clone(),
                index,
                bounds: mon.bounds,
                work_area: mon.os_work_area,
                scale: mon.scale,
            })
            .collect()
    }

    /// The monitors as [`places`](Self::places) will list them once
    /// `displays` are taken in as a display change, each with the id it
    /// will have then, so a bar can say how far it reaches into them in
    /// the same step and windows move once.
    pub fn places_for(&self, displays: &[Display]) -> Vec<MonitorPlace> {
        let pairs = pair_monitors(&self.monitors, displays);
        let ids = stable_ids(&self.monitors, displays, &pairs, &self.reconnect);
        (displays.iter().zip(ids).enumerate())
            .map(|(index, (display, id))| MonitorPlace {
                id,
                index,
                bounds: display.bounds,
                work_area: display.work_area,
                scale: display.scale,
            })
            .collect()
    }
}

/// One connected monitor, as a bar needs to know it to lay itself out
/// and say how far it reaches. The tiler's own light list, so the space
/// is worked out whether or not anything builds the bar API's state.
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorPlace {
    /// The id the monitor keeps through display changes.
    pub id: StableMonitorId,
    /// Position from the left, from 0, as `bar.toml`'s `monitors` counts.
    pub index: usize,
    /// The whole monitor.
    pub bounds: Rect,
    /// What the system leaves for windows, before any bar is kept clear
    /// of.
    pub work_area: Rect,
    /// Scale factor, 1.0 at 100%.
    pub scale: f64,
}

/// How far the active bar reaches into each monitor, which tiles keep
/// clear of.
///
/// Monitors are named by [`StableMonitorId`], so a reach means the same
/// display after a display change. A reach is measured from the monitor's
/// edge, not from the system's work area, and tiles keep clear of whichever
/// of the two reaches further, so a bar never needs to know what the
/// system keeps. Only the top edge is used. A monitor with no entry has
/// nothing kept clear.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Reach {
    tops: Vec<(StableMonitorId, i32)>,
}

impl Reach {
    /// A reach of nothing on every monitor.
    pub fn none() -> Self {
        Self::default()
    }

    /// The reach of a bar `height` pixels tall drawn at the top of the
    /// system's work area, on the monitors at `indices` in `places`, or on
    /// every one when `indices` is empty.
    pub fn bar_on(places: &[MonitorPlace], height: i32, indices: &[usize]) -> Self {
        let mut reach = Self::none();
        for place in places {
            if indices.is_empty() || indices.contains(&place.index) {
                let gap = place.work_area.y - place.bounds.y;
                reach.set_top(place.id.clone(), gap + height);
            }
        }
        reach
    }

    /// Has the bar reach `pixels` down from the top edge of monitor `id`,
    /// in place of what it reached there. Negative counts as 0.
    pub fn set_top(&mut self, id: StableMonitorId, pixels: i32) {
        let pixels = pixels.max(0);
        match self.tops.iter_mut().find(|(at, _)| *at == id) {
            Some((_, top)) => *top = pixels,
            None => self.tops.push((id, pixels)),
        }
    }

    /// How far the bar reaches down from the top edge of monitor `id`.
    pub fn top(&self, id: &StableMonitorId) -> i32 {
        (self.tops.iter())
            .find(|(at, _)| at == id)
            .map_or(0, |&(_, top)| top)
    }
}

#[cfg(test)]
#[path = "tests/reach_tests.rs"]
mod tests;

//! `[bar.reserve]` in `config.toml`: space kept for a bar another program
//! draws, for a bar that cannot say how far it reaches.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::limits::BAR_RESERVE_MAX;

/// Space kept clear of tiles on each edge of every monitor, in pixels at
/// 100%, for a bar another program draws. Each monitor's scale multiplies
/// it, so one value fits monitors of mixed scale.
///
/// It is kept whether the bar runs or not, and only while the bar is not
/// mosaico's own.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BarReserve {
    /// Kept down from the top edge.
    pub top: i32,
    /// Kept up from the bottom edge.
    pub bottom: i32,
    /// Kept in from the left edge.
    pub left: i32,
    /// Kept in from the right edge.
    pub right: i32,
    /// Monitors that keep space of their own in place of the edges above,
    /// each named by its position from the left, from 0, or by the id
    /// `mosaico debug list` shows.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub monitor: BTreeMap<String, ReserveEdges>,
}

/// The space one monitor keeps, in `[bar.reserve.monitor.<name>]`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReserveEdges {
    /// Kept down from the top edge.
    pub top: i32,
    /// Kept up from the bottom edge.
    pub bottom: i32,
    /// Kept in from the left edge.
    pub left: i32,
    /// Kept in from the right edge.
    pub right: i32,
}

impl BarReserve {
    /// Brings every edge into 0 to [`BAR_RESERVE_MAX`], as loading does.
    pub fn validate(&mut self) {
        for edge in [
            &mut self.top,
            &mut self.bottom,
            &mut self.left,
            &mut self.right,
        ] {
            *edge = (*edge).clamp(0, BAR_RESERVE_MAX);
        }
        self.monitor.values_mut().for_each(ReserveEdges::validate);
    }

    /// The space the monitor at `index` from the left, with the id `id`,
    /// keeps: its own table, found by id first, or else the edges every
    /// monitor keeps.
    pub fn for_monitor(&self, index: usize, id: &str) -> ReserveEdges {
        (self.monitor.get(id))
            .or_else(|| self.monitor.get(&index.to_string()))
            .copied()
            .unwrap_or(ReserveEdges {
                top: self.top,
                bottom: self.bottom,
                left: self.left,
                right: self.right,
            })
    }
}

impl ReserveEdges {
    fn validate(&mut self) {
        for edge in [
            &mut self.top,
            &mut self.bottom,
            &mut self.left,
            &mut self.right,
        ] {
            *edge = (*edge).clamp(0, BAR_RESERVE_MAX);
        }
    }
}

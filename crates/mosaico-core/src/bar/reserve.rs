//! The space `[bar.reserve]` keeps on each monitor, for a bar that cannot
//! say how far it reaches.

use crate::config::{BarReserve, ReserveEdges};
use crate::tiler::{Edges, MonitorPlace, Reach, Side};

/// What `[bar.reserve]` keeps on one monitor.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservedSpace {
    /// The space kept from each edge of the monitor, in pixels at its
    /// scale.
    pub edges: Edges,
    /// The edges whose space was refused because it would take more than
    /// a quarter of the monitor, which keep nothing.
    pub refused: Vec<Side>,
}

impl ReservedSpace {
    /// What `reserve` keeps on the monitor at `place`: its own table or
    /// the edges every monitor keeps, multiplied by its scale, with no
    /// edge past a quarter of the monitor.
    pub fn on(reserve: &BarReserve, place: &MonitorPlace) -> Self {
        let ReserveEdges {
            top,
            bottom,
            left,
            right,
        } = reserve.for_monitor(place.index, place.id.as_str());
        let edges = Edges::new(top, bottom, left, right).scaled(place.scale);
        Self {
            refused: edges.too_large(&place.bounds),
            edges: edges.within(&place.bounds),
        }
    }
}

/// How far `reserve` reaches into each of `places`, so tiles keep clear of
/// a bar mosaico cannot ask.
pub fn reserve_reach(reserve: &BarReserve, places: &[MonitorPlace]) -> Reach {
    let mut reach = Reach::none();
    for place in places {
        let space = ReservedSpace::on(reserve, place);
        if !space.edges.is_none() {
            reach.set(place.id.clone(), space.edges);
        }
    }
    reach
}

/// The names of `[bar.reserve.monitor.<name>]` tables that match none of
/// `places`, by id or by position, so they keep nothing.
pub fn unmatched_monitors<'a>(reserve: &'a BarReserve, places: &[MonitorPlace]) -> Vec<&'a str> {
    let matches = |name: &str| {
        (places.iter()).any(|place| place.id.as_str() == name || place.index.to_string() == name)
    };
    (reserve.monitor.keys())
        .map(String::as_str)
        .filter(|name| !matches(name))
        .collect()
}

#[cfg(test)]
#[path = "tests/reserve_tests.rs"]
mod tests;

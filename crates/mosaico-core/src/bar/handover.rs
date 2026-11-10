//! The space the previous bar kept, held for a moment after a switch to a
//! bar another program draws, which only says how far it reaches once it
//! connects.

use std::time::{Duration, Instant};

use crate::tiler::{Edges, MonitorPlace, Reach, Side};

use super::BarProvider;

/// How long a switch holds the previous bar's space for a bar that is yet
/// to say how far it reaches.
pub const HANDOVER: Duration = Duration::from_secs(5);

/// The edges the previous bar kept that the new one does not keep yet.
///
/// Without it, a switch from mosaico's bar to another program's moves
/// every tile up into the space the old bar left, then down again a moment
/// later as the new bar connects and asks for the same space. Holding it
/// moves them once, or not at all.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Handover {
    held: Reach,
    until: Instant,
}

impl Handover {
    /// Holds, until `until`, each edge `old` kept on a monitor that `new`
    /// keeps nothing of. `None` when there is no such edge.
    pub fn begin(old: &Reach, new: &Reach, until: Instant) -> Option<Self> {
        let mut held = Reach::none();
        for (id, kept) in old.monitors() {
            let now = new.edges(id);
            let mut edges = Edges::default();
            for side in Side::ALL {
                if now.get(side) == 0 {
                    edges.set(side, kept.get(side));
                }
            }
            if !edges.is_none() {
                held.set(id.clone(), edges);
            }
        }
        (!held.is_none()).then_some(Self { held, until })
    }

    /// Lets go of every monitor `new` now reaches into, which the new bar
    /// has taken over.
    pub fn release(&mut self, new: &Reach) {
        let mut held = Reach::none();
        for (id, edges) in self.held.monitors() {
            if new.edges(id).is_none() {
                held.set(id.clone(), edges);
            }
        }
        self.held = held;
    }

    /// Whether there is nothing left to hold at `now`.
    pub fn over(&self, now: Instant) -> bool {
        now >= self.until || self.held.is_none()
    }

    /// The space tiles keep clear of while `new` is the bar's own reach.
    pub fn with(&self, new: &Reach) -> Reach {
        new.furthest(&self.held)
    }
}

/// How far `provider` reaches into `places`, with the space `handover`
/// still holds, which lets go of the monitors the provider reaches into.
pub(super) fn reach_of(
    provider: &dyn BarProvider,
    handover: &mut Option<Handover>,
    places: &[MonitorPlace],
) -> Reach {
    let reach = provider.reach(places);
    match handover {
        Some(held) => {
            held.release(&reach);
            held.with(&reach)
        }
        None => reach,
    }
}

#[cfg(test)]
#[path = "tests/handover_tests.rs"]
mod tests;

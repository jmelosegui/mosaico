//! The space bars reserve through the bar API.

use std::time::{Duration, Instant};

use crate::api::{MonitorTarget, Peer};
use crate::tiler::{Edges, MonitorPlace, Reach};

/// How long the space of a session that ended is kept, so a bar that
/// restarts, or reconnects after a daemon hiccup, does not move every tile
/// out and back.
pub const RESERVATION_GRACE: Duration = Duration::from_secs(5);

/// One reservation: a session's space on the monitors it names.
#[derive(Debug, Clone, PartialEq)]
struct Held {
    session: u64,
    bar: Option<String>,
    process: Option<u32>,
    target: MonitorTarget,
    edges: Edges,
    /// When it ends, once its session has.
    until: Option<Instant>,
}

/// The space bars reserve through the bar API, kept by session and by the
/// monitors each reservation names.
///
/// A reservation names a monitor by its id, so it is kept while that
/// monitor is disconnected and applies again when it comes back. Several
/// reservations on one edge keep clear the furthest of them.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Reservations {
    held: Vec<Held>,
}

impl Reservations {
    /// Has `peer` reserve `edges` on `target`, in place of what it had
    /// there. No edges at all ends that reservation.
    pub fn set(&mut self, peer: &Peer, target: MonitorTarget, edges: Edges) {
        self.held
            .retain(|held| held.session != peer.session || held.target != target);
        if !edges.is_none() {
            self.held.push(Held {
                session: peer.session,
                bar: peer.bar.clone(),
                process: peer.process,
                target,
                edges,
                until: None,
            });
        }
    }

    /// Ends what `session` reserved on `target`, or all it reserved for
    /// `None`. Whether that ended any.
    pub fn release(&mut self, session: u64, target: Option<&MonitorTarget>) -> bool {
        let before = self.held.len();
        self.held.retain(|held| {
            held.session != session || target.is_some_and(|target| held.target != *target)
        });
        self.held.len() != before
    }

    /// `session` ended: what it reserved is kept until `until`.
    pub fn end(&mut self, session: u64, until: Instant) {
        for held in self.held.iter_mut().filter(|held| held.session == session) {
            held.until = Some(until);
        }
    }

    /// Lets go of what is held past `now`. Whether that let go of any.
    pub fn expire(&mut self, now: Instant) -> bool {
        let before = self.held.len();
        self.held
            .retain(|held| held.until.is_none_or(|until| now < until));
        self.held.len() != before
    }

    /// When the next reservation of an ended session runs out.
    pub fn due(&self) -> Option<Instant> {
        self.held.iter().filter_map(|held| held.until).min()
    }

    /// How far the reservations reach into `places`: on each edge, the
    /// furthest of them, with what reaches past a quarter of a monitor
    /// left out.
    pub fn reach(&self, places: &[MonitorPlace]) -> Reach {
        let mut reach = Reach::none();
        for place in places {
            let edges = (self.held.iter())
                .filter(|held| held.target.covers(&place.id))
                .fold(Edges::default(), |edges, held| edges.furthest(held.edges));
            if !edges.is_none() {
                reach.set(place.id.clone(), edges.within(&place.bounds));
            }
        }
        reach
    }

    /// The processes of the bars holding space, each once.
    pub fn processes(&self) -> Vec<u32> {
        let mut processes = Vec::new();
        for process in self.held.iter().filter_map(|held| held.process) {
            if let Err(at) = processes.binary_search(&process) {
                processes.insert(at, process);
            }
        }
        processes
    }

    /// Keeps only the reservations of the bars `keep` accepts, by the name
    /// the bar gave. Whether that let go of any.
    pub fn retain(&mut self, keep: impl Fn(Option<&str>) -> bool) -> bool {
        let before = self.held.len();
        self.held.retain(|held| keep(held.bar.as_deref()));
        self.held.len() != before
    }

    /// Lets go of every reservation.
    pub fn clear(&mut self) {
        self.held.clear();
    }
}

#[cfg(test)]
#[path = "tests/reservations_tests.rs"]
mod tests;

//! The space bars reserve through the bar API, as the bar host keeps the
//! tiles clear of it and leaves those bars' windows untiled.

use std::time::Instant;

use crate::api::{ApiError, ErrorCode, MonitorTarget, Peer, Reserve};
use crate::tiler::{MonitorPlace, Platform, Reach, TilingManager};

use super::super::handover::reach_of;
use super::super::reservations::RESERVATION_GRACE;
use super::BarHost;

impl BarHost {
    /// Has the session `peer` keep `reserve`'s space clear of tiles, in
    /// place of what it reserved on the same monitors, then reads the work
    /// areas again, as the bar may also have told the system of its space.
    /// The windows of the bar's process are no longer tiled.
    ///
    /// Refused when the provider does not accept the bar, when it names a
    /// monitor that is not connected, or when it reaches past a quarter of
    /// a monitor on an edge.
    pub fn reserve<P: Platform>(
        &mut self,
        manager: &mut TilingManager<P>,
        peer: &Peer,
        reserve: &Reserve,
    ) -> Result<(), ApiError> {
        if !self.provider.accepts(peer.bar.as_deref()) {
            let name = self.provider.name();
            let message = format!(
                "the bar provider is `{name}`, which keeps its own space; set `[bar] provider` \
                 to `external` for this bar to reserve its own"
            );
            return Err(ApiError::new(ErrorCode::NotProvider, message));
        }
        let places = manager.places();
        if let MonitorTarget::One(id) = &reserve.monitor
            && !places.iter().any(|place| place.id == *id)
        {
            let message = format!("no monitor {} is connected", id.as_str());
            return Err(ApiError::new(ErrorCode::NoMonitor, message));
        }
        let edges = reserve.edges();
        for place in places.iter().filter(|p| reserve.monitor.covers(&p.id)) {
            if let Some(side) = edges.too_large(&place.bounds).first() {
                let message = format!(
                    "`{}` reaches past a quarter of monitor {}",
                    side.name(),
                    place.id.as_str()
                );
                return Err(ApiError::new(ErrorCode::TooLarge, message));
            }
        }
        self.reservations.set(peer, reserve.monitor.clone(), edges);
        self.tell_processes(manager);
        if self.reset_work_areas(manager).is_err() {
            self.keep_clear(manager, &manager.places());
        }
        Ok(())
    }

    /// Ends what `session` reserved on `target`, or all it reserved for
    /// `None`, at once.
    pub fn release<P: Platform>(
        &mut self,
        manager: &mut TilingManager<P>,
        session: u64,
        target: Option<&MonitorTarget>,
    ) {
        if self.reservations.release(session, target) {
            self.space_changed(manager);
        }
    }

    /// The session `session` ended at `now`: what it reserved is kept for
    /// [`RESERVATION_GRACE`], so a bar that restarts takes it over without
    /// the tiles moving out and back.
    pub fn session_ended(&mut self, session: u64, now: Instant) {
        self.reservations.end(session, now + RESERVATION_GRACE);
    }

    /// How far the provider's bars and the reservations reach into
    /// `places`.
    pub(super) fn wanted(&self, places: &[MonitorPlace]) -> Reach {
        let reach = self.provider.reach(places);
        reach.furthest(&self.reservations.reach(places))
    }

    /// Takes in a change to the space reserved: tells the tiler the bars'
    /// processes and keeps the tiles clear of the space.
    pub(super) fn space_changed<P: Platform>(&mut self, manager: &mut TilingManager<P>) {
        self.tell_processes(manager);
        self.keep_clear(manager, &manager.places());
    }

    /// Tells the tiler the processes of the bars that reserve space, when
    /// they changed.
    pub(super) fn tell_processes<P: Platform>(&mut self, manager: &mut TilingManager<P>) {
        let processes = self.reservations.processes();
        if processes != self.processes {
            self.processes = processes.clone();
            manager.set_bar_processes(processes);
        }
    }

    /// Keeps the tiles clear of how far the bars reach into `places` now,
    /// moving them only when that changed.
    pub(super) fn keep_clear<P: Platform>(
        &mut self,
        manager: &mut TilingManager<P>,
        places: &[MonitorPlace],
    ) {
        let reach = reach_of(self.wanted(places), &mut self.handover);
        if reach != self.reach {
            self.reach = reach;
            manager.set_reach(self.reach.clone());
        }
    }
}

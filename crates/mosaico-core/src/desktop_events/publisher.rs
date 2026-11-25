//! Turning the daemon's steps into events for its subscribers.

use crate::ipc::{Response, Subscribers};
use crate::snapshot::DesktopSnapshot;

use super::{DesktopEvent, EventKind, changes};

/// Decides what to tell subscribers after each daemon step.
///
/// It keeps the desktop as subscribers last saw it, and only while
/// someone is subscribed, so a daemon nobody listens to never builds a
/// snapshot.
pub struct Publisher {
    subscribers: Subscribers,
    /// The desktop as subscribers last heard it, while there are any.
    last: Option<DesktopSnapshot>,
    paused: bool,
}

impl Publisher {
    /// A publisher for `subscribers`, with the hotkeys not paused.
    pub fn new(subscribers: Subscribers) -> Self {
        Self {
            subscribers,
            last: None,
            paused: false,
        }
    }

    /// Answers a subscription to `kinds` with `now`, the desktop at this
    /// moment, and opens it, so the subscriber hears every change after
    /// `now`. Subscribers already open first hear what changed since
    /// they last heard, so `now` is where all of them stand.
    pub fn subscribe(&mut self, kinds: Vec<EventKind>, now: DesktopSnapshot) -> Response {
        self.catch_up(&now);
        self.subscribers.open(kinds);
        let response = Response::ok().with_data(&now);
        self.last = Some(now);
        response
    }

    /// Tells subscribers what changed in the step that just ran, given
    /// whether the hotkeys are paused now and a way to see the desktop.
    pub fn after_step(&mut self, paused: bool, now: impl FnOnce() -> DesktopSnapshot) {
        let was_paused = std::mem::replace(&mut self.paused, paused);
        if self.subscribers.is_empty() {
            self.last = None;
            return;
        }
        if was_paused != paused {
            self.subscribers.publish(&[DesktopEvent::Paused { paused }]);
        }
        let now = now();
        self.catch_up(&now);
        self.last = Some(now);
    }

    /// Tells subscribers the daemon loaded a changed configuration.
    pub fn config_reloaded(&self) {
        if !self.subscribers.is_empty() {
            self.subscribers.publish(&[DesktopEvent::ConfigReloaded]);
        }
    }

    /// Ends every subscription, as when the daemon stops.
    pub fn close(&mut self) {
        self.subscribers.close();
        self.last = None;
    }

    fn catch_up(&self, now: &DesktopSnapshot) {
        if let Some(last) = &self.last {
            let events = changes(last, now);
            if !events.is_empty() {
                self.subscribers.publish(&events);
            }
        }
    }
}

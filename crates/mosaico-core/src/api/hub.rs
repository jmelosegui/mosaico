//! The state the bars are served, shared by the daemon loop and the
//! sessions.
//!
//! The loop publishes the latest state and never waits for a session.
//! Each session's writer takes the latest state when it can write and
//! sends what changed since its previous message, so a bar that reads
//! slowly gets fewer, larger updates, and nothing piles up.

use std::collections::BTreeMap;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::outbox::Outbox;
use super::{ApiError, BarSession, ErrorCode, Limits, LoopRequest, State, Stopper};

/// Sends a request to the daemon loop; `false` once the loop is gone.
pub type ToLoop = Box<dyn Fn(LoopRequest) -> bool + Send + Sync>;

/// The state the bars are served and the sessions open.
pub struct Hub {
    shared: Mutex<Shared>,
    pub(super) changed: Condvar,
    boot: String,
    pub(super) limits: Limits,
    to_loop: ToLoop,
}

pub(super) struct Shared {
    pub(super) state: Arc<State>,
    /// Counts the states published that differed from the one before.
    pub(super) rev: u64,
    /// Whether `state` was published since sessions were last followed.
    pub(super) fresh: bool,
    pub(super) connections: usize,
    pub(super) sessions: BTreeMap<u64, Outbox>,
    /// What each open session said of itself in `hello`, oldest first.
    pub(super) about: Vec<BarSession>,
    pub(super) next_session: u64,
    pub(super) closing: Option<String>,
    pub(super) stoppers: Vec<Stopper>,
}

/// One connection, counted while it lives.
pub(super) struct Ticket {
    hub: Arc<Hub>,
    session: Option<u64>,
}

impl Hub {
    /// A hub with nothing published, for a daemon whose start `boot`
    /// names, that reaches its loop through `to_loop`.
    pub fn new(boot: String, limits: Limits, to_loop: ToLoop) -> Arc<Self> {
        Arc::new(Self {
            shared: Mutex::new(Shared {
                state: Arc::default(),
                rev: 0,
                fresh: false,
                connections: 0,
                sessions: BTreeMap::new(),
                about: Vec::new(),
                next_session: 1,
                closing: None,
                stoppers: Vec::new(),
            }),
            changed: Condvar::new(),
            boot,
            limits,
            to_loop,
        })
    }

    /// Names the daemon's start.
    pub fn boot(&self) -> &str {
        &self.boot
    }

    /// How far sessions may go.
    pub fn limits(&self) -> Limits {
        self.limits
    }

    /// Makes `state` the latest, a new revision when it differs.
    pub fn publish(&self, state: &State) {
        let mut shared = self.lock();
        if *shared.state != *state {
            shared.state = Arc::new(state.clone());
            shared.rev += 1;
        }
        shared.fresh = true;
        self.changed.notify_all();
    }

    /// Whether any session follows the state, so it should be published.
    pub fn watched(&self) -> bool {
        !self.lock().sessions.is_empty()
    }

    /// Whether the state was published since sessions were last followed.
    /// A state built while nobody followed it was not published, so the
    /// hub's may be old until it is.
    pub fn fresh(&self) -> bool {
        self.lock().fresh
    }

    /// The latest state's revision.
    pub fn rev(&self) -> u64 {
        self.lock().rev
    }

    /// How many sessions are open.
    pub fn sessions(&self) -> usize {
        self.lock().sessions.len()
    }

    /// The sessions open, in the order they opened, as each described
    /// itself in `hello`. The space they reserve is the loop's to say.
    pub fn listed(&self) -> Vec<BarSession> {
        self.lock().about.clone()
    }

    /// Ends every session, each told `reason`, stops the listeners, and
    /// refuses what comes after.
    pub fn close(&self, reason: &str) {
        let stoppers = {
            let mut shared = self.lock();
            shared.closing.get_or_insert_with(|| reason.to_string());
            std::mem::take(&mut shared.stoppers)
        };
        self.changed.notify_all();
        stoppers.iter().for_each(|stop| stop());
    }

    /// Waits up to `timeout` for every connection to end; tells whether
    /// they did.
    pub fn wait_ended(&self, timeout: Duration) -> bool {
        let until = Instant::now() + timeout;
        let mut shared = self.lock();
        while shared.connections > 0 {
            let left = until.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            let waited = self.changed.wait_timeout(shared, left);
            shared = waited.map(|(g, _)| g).unwrap_or_else(|e| e.into_inner().0);
        }
        true
    }

    /// Stops `stop`'s listener on [`close`](Self::close), or now if closed.
    pub(super) fn add_stopper(&self, stop: Stopper) {
        let mut shared = self.lock();
        if shared.closing.is_none() {
            shared.stoppers.push(stop);
        } else {
            drop(shared);
            stop();
        }
    }

    /// Counts a new connection, or `None` when too many are open or the
    /// hub is closing.
    pub(super) fn admit(self: &Arc<Self>) -> Option<Ticket> {
        let mut shared = self.lock();
        if shared.closing.is_some() || shared.connections >= self.limits.connections {
            return None;
        }
        shared.connections += 1;
        Some(Ticket {
            hub: Arc::clone(self),
            session: None,
        })
    }

    /// Opens a session on `ticket`'s connection for the program `about`
    /// describes, and asks the loop for the state if nobody followed it
    /// yet.
    pub(super) fn join(&self, ticket: &mut Ticket, about: BarSession) -> Result<u64, ApiError> {
        let mut shared = self.lock();
        if shared.closing.is_some() {
            return Err(ApiError::new(ErrorCode::Stopping, "the daemon is stopping"));
        }
        if shared.sessions.len() >= self.limits.sessions {
            let message = format!("{} sessions are open already", shared.sessions.len());
            return Err(ApiError::new(ErrorCode::Busy, message));
        }
        let session = shared.next_session;
        shared.next_session += 1;
        shared.sessions.insert(session, Outbox::default());
        shared.about.push(BarSession { session, ..about });
        ticket.session = Some(session);
        let stale = !shared.fresh;
        drop(shared);
        if stale {
            self.ask(LoopRequest::Publish);
        }
        Ok(session)
    }

    /// Sends `request` to the loop; `false` once closing or the loop is
    /// gone.
    pub(super) fn ask(&self, request: LoopRequest) -> bool {
        if self.lock().closing.is_some() {
            return false;
        }
        (self.to_loop)(request)
    }

    pub(super) fn lock(&self) -> MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn wait<'a>(&self, guard: MutexGuard<'a, Shared>) -> MutexGuard<'a, Shared> {
        self.changed
            .wait(guard)
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl Ticket {
    /// The hub the connection counts on.
    pub(super) fn hub(&self) -> &Arc<Hub> {
        &self.hub
    }
}

impl Drop for Ticket {
    fn drop(&mut self) {
        let mut shared = self.hub.lock();
        shared.connections = shared.connections.saturating_sub(1);
        if let Some(session) = self.session {
            shared.sessions.remove(&session);
            shared.about.retain(|about| about.session != session);
            if shared.sessions.is_empty() {
                shared.fresh = false;
            }
        }
        self.hub.changed.notify_all();
    }
}

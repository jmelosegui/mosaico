//! What the sessions ask of the daemon loop, and how it answers.

use std::fmt;
use std::sync::Arc;

use crate::{Action, StableMonitorId};

use super::{ApiError, ErrorCode, Hub, Release, Reserve};

/// What a session asks of the daemon loop, which alone may touch the
/// tiler.
#[derive(Debug)]
pub enum LoopRequest {
    /// Publish the state now: a session opened while nobody was
    /// following it, so the hub's may be old.
    Publish,
    /// Runs an action from a bar on `monitor`, or on the focused monitor
    /// when `None`, as a click on that monitor's bar runs it, shows the
    /// bars the state, then answers `reply`.
    Run {
        /// What to run. Its scope was checked: it acts on a monitor or the
        /// daemon, never the focused window.
        action: Action,
        /// The monitor, by id.
        monitor: Option<StableMonitorId>,
        /// Where the answer goes.
        reply: Reply,
    },
    /// Keeps tiles clear of the space `reserve` asks for, for as long as
    /// `peer`'s session lasts, then answers `reply`.
    Reserve {
        /// Who asks.
        peer: Peer,
        /// The space, as the bar asked for it.
        reserve: Reserve,
        /// Where the answer goes.
        reply: Reply,
    },
    /// Ends the reservations `release` names of `session`, then answers
    /// `reply`.
    Release {
        /// The session they belong to.
        session: u64,
        /// Which ones.
        release: Release,
        /// Where the answer goes.
        reply: Reply,
    },
    /// Session `session`, which reserved space, ended. What it still
    /// holds is kept a moment longer, for a bar that starts again.
    Ended {
        /// The session.
        session: u64,
    },
}

/// The session a request comes from, and the bar it speaks for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Peer {
    /// The session.
    pub session: u64,
    /// The bar its `hello` named, such as `"yasb"`, if any.
    pub bar: Option<String>,
    /// The process that draws the bar, if known: the one `hello` named,
    /// or else the one at the other end of the connection.
    pub process: Option<u32>,
}

/// Answers one request. Answering after the state the request changed
/// was published makes the session send that state before the answer.
///
/// A reply dropped unanswered answers `stopping`, so a session never
/// waits for a request the daemon let go of.
pub struct Reply {
    hub: Option<Arc<Hub>>,
    session: u64,
    id: u64,
}

impl Reply {
    pub(super) fn new(hub: Arc<Hub>, session: u64, id: u64) -> Self {
        Self {
            hub: Some(hub),
            session,
            id,
        }
    }

    /// The request was carried out.
    pub fn ok(mut self) {
        self.answer(Ok(()));
    }

    /// The request failed with `code`, which `message` explains.
    pub fn fail(mut self, code: ErrorCode, message: impl Into<String>) {
        self.answer(Err(ApiError::new(code, message)));
    }

    fn answer(&mut self, result: Result<(), ApiError>) {
        if let Some(hub) = self.hub.take() {
            hub.answer(self.session, self.id, result);
        }
    }
}

impl Drop for Reply {
    fn drop(&mut self) {
        let message = "the daemon stopped before carrying it out";
        self.answer(Err(ApiError::new(ErrorCode::Stopping, message)));
    }
}

impl fmt::Debug for Reply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (f.debug_struct("Reply"))
            .field("session", &self.session)
            .field("id", &self.id)
            .finish()
    }
}

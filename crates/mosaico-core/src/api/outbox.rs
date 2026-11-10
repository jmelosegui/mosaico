//! What each session owes its bar beside the state, and how its writer
//! and reader wait on the hub.

use std::sync::Arc;

use super::hub::Hub;
use super::{ApiError, State, Topics};

/// What waits to be sent on one session, beside the state.
#[derive(Default)]
pub(super) struct Outbox {
    items: Vec<Out>,
    in_flight: usize,
    ended: bool,
}

/// One message a session's writer owes its bar, beside the state.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Out {
    /// A snapshot answering request `id`, of `topics` from now on when
    /// given.
    Snapshot { id: u64, topics: Option<Topics> },
    /// Request `id` was carried out, at revision `rev` when it may have
    /// changed the state.
    Ok { id: u64, rev: Option<u64> },
    /// A message was refused.
    Error { id: Option<u64>, error: ApiError },
    /// Ends the session once what came before is sent.
    End,
}

/// What a session's writer is to send next.
pub(super) struct Next {
    pub state: Arc<State>,
    pub rev: u64,
    /// Whether `state` was published, so it may be sent.
    pub fresh: bool,
    pub items: Vec<Out>,
    pub closing: Option<String>,
    pub ended: bool,
}

impl Hub {
    /// Has `session` send `out`.
    pub(super) fn push(&self, session: u64, out: Out) {
        if let Some(outbox) = self.lock().sessions.get_mut(&session) {
            outbox.items.push(out);
        }
        self.changed.notify_all();
    }

    /// Ends `session`: its writer stops, and its reader stops waiting.
    pub(super) fn end(&self, session: u64) {
        if let Some(outbox) = self.lock().sessions.get_mut(&session) {
            outbox.ended = true;
        }
        self.changed.notify_all();
    }

    /// Waits until `session` may have one more request with the loop and
    /// counts it; `false` once the session ends or the hub closes.
    pub(super) fn take_slot(&self, session: u64) -> bool {
        let mut shared = self.lock();
        loop {
            if shared.closing.is_some() {
                return false;
            }
            let Some(outbox) = shared.sessions.get_mut(&session) else {
                return false;
            };
            if outbox.ended {
                return false;
            }
            if outbox.in_flight < self.limits.in_flight {
                outbox.in_flight += 1;
                return true;
            }
            shared = self.wait(shared);
        }
    }

    /// Answers request `id` of `session`, at the latest revision.
    pub(super) fn answer(&self, session: u64, id: u64, result: Result<(), ApiError>) {
        let mut shared = self.lock();
        let rev = shared.rev;
        if let Some(outbox) = shared.sessions.get_mut(&session) {
            outbox.in_flight = outbox.in_flight.saturating_sub(1);
            outbox.items.push(match result {
                Ok(()) => Out::Ok { id, rev: Some(rev) },
                Err(error) => Out::Error {
                    id: Some(id),
                    error,
                },
            });
        }
        self.changed.notify_all();
    }

    /// Waits until `session` has something to send: a state other than
    /// revision `sent` once one is published, a message, or its end or
    /// the hub's. What is left to send once it ends is still handed out.
    /// Nothing but `bye` may be sent before the first state is published.
    pub(super) fn next(&self, session: u64, sent: Option<u64>) -> Next {
        let mut shared = self.lock();
        loop {
            let (rev, fresh, closing) = (shared.rev, shared.fresh, shared.closing.clone());
            let Some(outbox) = shared.sessions.get_mut(&session) else {
                return Next::ended(&shared.state, rev);
            };
            let pending = sent != Some(rev) || !outbox.items.is_empty();
            if outbox.ended || closing.is_some() || (fresh && pending) {
                let (items, ended) = (std::mem::take(&mut outbox.items), outbox.ended);
                let state = Arc::clone(&shared.state);
                return Next {
                    state,
                    rev,
                    fresh,
                    items,
                    closing,
                    ended,
                };
            }
            shared = self.wait(shared);
        }
    }
}

impl Next {
    fn ended(state: &Arc<State>, rev: u64) -> Self {
        Self {
            state: Arc::clone(state),
            rev,
            fresh: false,
            items: Vec::new(),
            closing: None,
            ended: true,
        }
    }
}

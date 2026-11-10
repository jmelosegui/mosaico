//! A session's writer: sends its bar the state as it changes, and the
//! replies, from a thread of its own, so a bar that reads slowly holds up
//! nobody else.

use crate::ipc::IpcError;
use crate::ipc::framing::write_message;

use super::hub::Hub;
use super::outbox::Out;
use super::{
    API_VERSION, Ack, ApiError, ApiWriter, Bye, DaemonMessage, ErrorCode, State, Topics, Welcome,
};

/// One session's side of the messages to its bar.
pub(super) struct SessionWriter<'a, W: ApiWriter> {
    out: W,
    hub: &'a Hub,
    session: u64,
    topics: Topics,
    /// The followed topics of the state last sent, `None` before the
    /// first.
    sent: Option<State>,
    /// The revision of the state being sent, `None` before the first.
    rev: Option<u64>,
    seq: u64,
}

impl<'a, W: ApiWriter> SessionWriter<'a, W> {
    pub(super) fn new(out: W, hub: &'a Hub, session: u64, topics: Topics) -> Self {
        Self {
            out,
            hub,
            session,
            topics,
            sent: None,
            rev: None,
            seq: 0,
        }
    }

    /// Sends `welcome` and a snapshot once the state is published, then
    /// what changes and the replies, until the session or the hub ends.
    /// Ends the session and the connection when done.
    pub(super) fn run(mut self) {
        if let Err(e) = self.serve() {
            crate::log_debug!("Bar API: session {} ended: {e}", self.session);
        }
        self.hub.end(self.session);
        self.out.shutdown();
    }

    fn serve(&mut self) -> Result<(), IpcError> {
        loop {
            let next = self.hub.next(self.session, self.rev);
            if !next.fresh || (next.ended && next.items.is_empty()) {
                return match next.closing.filter(|_| !next.ended) {
                    Some(reason) => self.send(&DaemonMessage::Bye {
                        bye: Bye { reason },
                    }),
                    None => Ok(()),
                };
            }
            let first = self.rev.replace(next.rev).is_none();
            if first {
                self.welcome(&next.state)?;
            }
            self.sync(&next.state)?;
            for item in next.items {
                match item {
                    Out::Snapshot { id, topics } => {
                        self.topics = topics.unwrap_or(self.topics);
                        self.snapshot(Some(id), &next.state)?;
                    }
                    Out::Ok { id, rev } => {
                        self.send(&DaemonMessage::Ok {
                            id,
                            ok: Ack { rev },
                        })?;
                    }
                    Out::Error { id, error } => self.send(&DaemonMessage::Error { id, error })?,
                    Out::End => return Ok(()),
                }
            }
            if let Some(reason) = next.closing {
                return self.send(&DaemonMessage::Bye {
                    bye: Bye { reason },
                });
            }
            if next.ended {
                return Ok(());
            }
        }
    }

    fn welcome(&mut self, state: &State) -> Result<(), IpcError> {
        let welcome = Welcome {
            api: API_VERSION,
            mosaico: state.daemon.version.clone(),
            boot: self.hub.boot().to_string(),
            provider: state.daemon.provider.clone(),
            topics: self.topics.names(),
        };
        self.send(&DaemonMessage::Welcome { welcome })
    }

    /// Sends what changed of the followed topics since the state last
    /// sent, if anything did, or a snapshot when none was sent.
    fn sync(&mut self, state: &State) -> Result<(), IpcError> {
        let Some(sent) = &self.sent else {
            return self.snapshot(None, state);
        };
        let latest = self.topics.filter(state);
        let update = sent.diff(&latest);
        if update.is_empty() {
            return Ok(());
        }
        self.sent = Some(latest);
        self.seq += 1;
        let (seq, rev) = (self.seq, self.rev.unwrap_or_default());
        self.send(&DaemonMessage::Update { seq, rev, update })
    }

    fn snapshot(&mut self, id: Option<u64>, state: &State) -> Result<(), IpcError> {
        self.sent = Some(self.topics.filter(state));
        self.seq += 1;
        let snapshot = self.topics.snapshot(state);
        let (seq, rev) = (self.seq, self.rev.unwrap_or_default());
        self.send(&DaemonMessage::Snapshot {
            id,
            seq,
            rev,
            snapshot,
        })
    }

    /// Writes `message` within the write limit. One too large to send is
    /// replaced by an error saying so, and ends the session.
    fn send(&mut self, message: &DaemonMessage) -> Result<(), IpcError> {
        self.out.set_write_timeout(self.hub.limits().write)?;
        match write_message(&mut self.out, message) {
            Err(IpcError::TooLarge) => {
                let error = ApiError::new(ErrorCode::TooLarge, "the state is too large to send");
                write_message(&mut self.out, &DaemonMessage::Error { id: None, error })?;
                Err(IpcError::TooLarge)
            }
            other => other,
        }
    }
}

/// Writes `message` to a bar before its session opens, then ends the
/// connection.
pub(super) fn refuse(out: &mut impl ApiWriter, hub: &Hub, message: &DaemonMessage) {
    if out.set_write_timeout(hub.limits().write).is_ok() {
        let _ = write_message(out, message);
        let _ = out.flush();
    }
    out.shutdown();
}

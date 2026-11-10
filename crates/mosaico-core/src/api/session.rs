//! The sessions: accepting bars, opening a session on `hello`, and
//! reading what each bar asks.

use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::thread;

use serde_json::Value;

use crate::Action;
use crate::action::Scope;
use crate::ipc::IpcError;
use crate::ipc::framing::read_message;

use super::hub::{Hub, Ticket};
use super::outbox::Out;
use super::writer::{SessionWriter, refuse};
use super::{
    ApiConnection, ApiError, ApiListener, ApiReader, ApiWriter, ClientMessage, DaemonMessage,
    ErrorCode, LoopRequest, Peer, Reply, Request, Run, SUPPORTED_APIS, Topics, negotiate,
};

/// Accepts bars on `listener` until `hub` closes, each session on threads
/// of its own. Returns once the listener stops or fails.
pub fn serve_api<L: ApiListener>(listener: &mut L, hub: &Arc<Hub>) {
    hub.add_stopper(listener.stopper());
    loop {
        let connection = match listener.accept() {
            Ok(Some(connection)) => connection,
            Ok(None) => return,
            Err(IpcError::Closed) => continue,
            Err(e) => {
                crate::log_error!("Bar API: no longer listening: {e}");
                return;
            }
        };
        // Past the limit, or while closing, the connection is dropped
        // unread: the daemon sends nothing before `hello`.
        let Some(ticket) = hub.admit() else {
            continue;
        };
        let spawned = (thread::Builder::new())
            .name("bar-session".into())
            .spawn(move || run_session(connection, ticket));
        if let Err(e) = spawned {
            crate::log_warn!("Bar API: could not start a session: {e}");
        }
    }
}

/// Serves one connection: waits for `hello`, opens the session, then
/// reads the bar's requests while a writer thread sends it the state.
fn run_session<C: ApiConnection>(connection: C, ticket: Ticket) {
    let process = connection.peer_process();
    let Ok((reader, mut writer)) = connection.split() else {
        return;
    };
    let hub = Arc::clone(ticket.hub());
    let mut reader = BufReader::new(reader);
    let _ = reader.get_mut().set_read_timeout(Some(hub.limits().hello));
    let Ok(first) = read_message::<Value>(&mut reader) else {
        return writer.shutdown();
    };
    let hello = match ClientMessage::parse(first) {
        Ok(ClientMessage::Hello { hello }) => hello,
        _ => {
            let error = ApiError::new(ErrorCode::Malformed, "a session starts with hello");
            return refuse(&mut writer, &hub, &DaemonMessage::Error { id: None, error });
        }
    };
    if negotiate(&hello.api).is_none() {
        let mut error = ApiError::new(ErrorCode::UnsupportedApi, "no API version in common");
        error.api = Some(SUPPORTED_APIS.to_vec());
        return refuse(&mut writer, &hub, &DaemonMessage::Error { id: None, error });
    }
    let mut ticket = ticket;
    let session = match hub.join(&mut ticket) {
        Ok(session) => session,
        Err(error) => return refuse(&mut writer, &hub, &DaemonMessage::Error { id: None, error }),
    };
    crate::log_info!(
        "Bar API: session {session} opened by {}",
        hello.client.as_deref().unwrap_or("a bar")
    );
    let _ = reader.get_mut().set_read_timeout(None);
    let topics = Topics::requested(hello.topics.as_deref());
    let peer = Peer {
        session,
        bar: hello.bar,
        process: hello.process.or(process),
    };
    thread::scope(|scope| {
        scope.spawn(|| SessionWriter::new(writer, &hub, session, topics).run());
        let reserved = read_requests(&mut reader, &hub, &peer);
        hub.end(session);
        if reserved {
            hub.ask(LoopRequest::Ended { session });
        }
    });
    crate::log_info!("Bar API: session {session} closed");
    drop(ticket);
}

/// Reads and takes in the bar's messages until the connection ends.
/// Tells whether the bar asked to reserve space meanwhile.
fn read_requests(reader: &mut impl BufRead, hub: &Arc<Hub>, peer: &Peer) -> bool {
    let session = peer.session;
    let mut reserved = false;
    loop {
        let refusal = match read_message::<Value>(reader) {
            Ok(value) => match ClientMessage::parse(value) {
                Ok(ClientMessage::Request { id, request }) => {
                    reserved |= matches!(request, Request::Reserve(_));
                    if !take(hub, peer, id, request) {
                        return reserved;
                    }
                    continue;
                }
                Ok(ClientMessage::Hello { .. }) => {
                    let message = "hello was already sent";
                    Out::Error {
                        id: None,
                        error: ApiError::new(ErrorCode::Malformed, message),
                    }
                }
                Err(refusal) => Out::Error {
                    id: refusal.id,
                    error: refusal.error,
                },
            },
            Err(IpcError::Malformed(e)) => Out::Error {
                id: None,
                error: ApiError::new(ErrorCode::Malformed, format!("not JSON: {e}")),
            },
            Err(IpcError::TooLarge) => {
                let message = "a message was too large; the session ends";
                let error = ApiError::new(ErrorCode::TooLarge, message);
                hub.push(session, Out::Error { id: None, error });
                hub.push(session, Out::End);
                return reserved;
            }
            Err(_) => return reserved,
        };
        hub.push(session, refusal);
    }
}

/// Takes in request `id`; `false` once the session can take no more.
fn take(hub: &Arc<Hub>, peer: &Peer, id: u64, request: Request) -> bool {
    let session = peer.session;
    let out = match request {
        Request::Run(run) => return take_run(hub, session, id, run),
        Request::Reserve(reserve) => {
            let peer = peer.clone();
            return hand_over(hub, session, id, |reply| LoopRequest::Reserve {
                peer,
                reserve,
                reply,
            });
        }
        Request::Release(release) => {
            return hand_over(hub, session, id, |reply| LoopRequest::Release {
                session,
                release,
                reply,
            });
        }
        Request::Subscribe(subscribe) => Out::Snapshot {
            id,
            topics: Some(Topics::requested(Some(&subscribe.topics))),
        },
        Request::Resync {} => Out::Snapshot { id, topics: None },
        Request::Ping {} => Out::Ok { id, rev: None },
    };
    hub.push(session, out);
    true
}

/// Checks `run` and hands it to the daemon loop, once the session has
/// fewer requests with the loop than it may.
fn take_run(hub: &Arc<Hub>, session: u64, id: u64, run: Run) -> bool {
    let refuse = |code, message: String| {
        let error = ApiError::new(code, message);
        hub.push(
            session,
            Out::Error {
                id: Some(id),
                error,
            },
        );
        true
    };
    let Ok(action) = run.action.parse::<Action>() else {
        return refuse(
            ErrorCode::UnknownAction,
            format!("unknown action `{}`", run.action),
        );
    };
    if !matches!(action.scope(), Scope::Monitor | Scope::Daemon) {
        let message = format!("`{action}` acts on the focused window, which a bar may not");
        return refuse(ErrorCode::NotAllowed, message);
    }
    let monitor = run.monitor;
    hand_over(hub, session, id, |reply| LoopRequest::Run {
        action,
        monitor,
        reply,
    })
}

/// Hands request `id` to the daemon loop as `request` makes it of its
/// reply, once the session has fewer requests with the loop than it may.
fn hand_over(
    hub: &Arc<Hub>,
    session: u64,
    id: u64,
    request: impl FnOnce(Reply) -> LoopRequest,
) -> bool {
    if !hub.take_slot(session) {
        return false;
    }
    let reply = Reply::new(Arc::clone(hub), session, id);
    // A request the loop never sees answers `stopping` as its reply drops.
    hub.ask(request(reply));
    true
}

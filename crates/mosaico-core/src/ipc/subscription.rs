//! A client's side of an event subscription.

use std::io::{BufRead, BufReader, Read};

use crate::desktop_events::{DesktopEvent, EventKind};
use crate::snapshot::DesktopSnapshot;

use super::framing::{read_message, write_message};
use super::{
    Command, IpcClient, IpcError, IpcStream, REPLY_TIMEOUT, Request, Response, ResponseStatus,
};

/// The daemon's answer to a subscription.
pub enum Subscribed {
    /// The daemon took it: the desktop as it was, then its events.
    Accepted(Subscription),
    /// The daemon refused it, and said why.
    Refused(Response),
}

/// An accepted subscription.
pub struct Subscription {
    /// The desktop when the subscription started. Every event that
    /// follows is a change from it.
    pub snapshot: DesktopSnapshot,
    /// What changes from here on.
    pub events: EventStream,
}

/// The events of a subscription, one at a time. It can move to another
/// thread, so a client can wait for events and for other things at once.
pub struct EventStream {
    reader: Box<dyn BufRead + Send>,
}

impl EventStream {
    /// Events read from `reader`, one JSON object per line, as the
    /// daemon writes them.
    pub fn new(reader: impl Read + Send + 'static) -> Self {
        Self {
            reader: Box::new(BufReader::new(reader)),
        }
    }

    /// Waits for the next event. Fails with [`IpcError::Closed`] when the
    /// daemon ends the subscription, as it does when it stops, and after
    /// it sent [`DesktopEvent::Lagged`].
    pub fn next_event(&mut self) -> Result<DesktopEvent, IpcError> {
        read_message(&mut self.reader)
    }
}

/// Subscribes to the events of `kinds`, or to every kind when it is
/// empty, and waits for the daemon's answer for up to [`REPLY_TIMEOUT`].
/// Once accepted, reading the events waits as long as it takes.
pub fn subscribe<C>(client: &C, kinds: Vec<EventKind>) -> Result<Subscribed, IpcError>
where
    C: IpcClient + ?Sized,
    C::Stream: Send + 'static,
{
    let mut stream = client.connect()?;
    let request = Request::new(Command::Subscribe { events: kinds });
    write_message(&mut stream, &request)?;
    stream.set_read_timeout(REPLY_TIMEOUT)?;
    let mut reader = BufReader::new(stream);
    let response: Response = read_message(&mut reader).map_err(|e| match e {
        IpcError::Io(e) if e.kind() == std::io::ErrorKind::TimedOut => IpcError::TimedOut,
        other => other,
    })?;
    if response.status != ResponseStatus::Ok {
        return Ok(Subscribed::Refused(response));
    }
    // An ok answer without the desktop is not one this build can use.
    let Some(snapshot) = response.data::<DesktopSnapshot>() else {
        return Ok(Subscribed::Refused(response));
    };
    reader.get_mut().clear_read_timeout()?;
    Ok(Subscribed::Accepted(Subscription {
        snapshot,
        events: EventStream {
            reader: Box::new(reader),
        },
    }))
}

#[cfg(test)]
#[path = "tests/subscription_tests.rs"]
mod tests;

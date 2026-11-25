//! `mosaico events`: prints what changes on the desktop as it happens.

use mosaico_core::desktop_events::{DesktopEvent, EventKind};
use mosaico_core::ipc::{IpcError, Subscribed};

use crate::commands::context::{Context, Exit, complain, say};
use crate::commands::request;

/// Subscribes to the events named in `kinds`, every kind when empty, and
/// prints each one until the daemon stops: as text, or as one JSON object
/// per line with `json`.
pub fn execute(cx: &mut Context, json: bool, kinds: &[String]) -> Exit {
    let mut wanted = Vec::with_capacity(kinds.len());
    for name in kinds {
        match name.parse::<EventKind>() {
            Ok(kind) => wanted.push(kind),
            Err(e) => {
                complain!(cx, "Error: {e}");
                return Exit::Failure;
            }
        }
    }
    let mut subscription = match cx.daemon.subscribe(wanted) {
        Ok(Subscribed::Accepted(subscription)) => subscription,
        Ok(Subscribed::Refused(response)) => {
            complain!(cx, "Error: {}", request::describe_refusal(&response));
            return Exit::Failure;
        }
        Err(e) => return request::unreachable_daemon(cx, &e),
    };
    loop {
        match subscription.events.next_event() {
            Ok(event) => {
                print(cx, &event, json);
                if event == DesktopEvent::Lagged {
                    complain!(
                        cx,
                        "Fell behind the daemon's events; run it again to go on."
                    );
                    return Exit::Failure;
                }
            }
            Err(IpcError::Closed) => return Exit::Success,
            Err(e) => {
                complain!(cx, "Lost the daemon's events: {e}");
                return Exit::Failure;
            }
        }
    }
}

fn print(cx: &mut Context, event: &DesktopEvent, json: bool) {
    if json {
        say!(cx, "{}", event.to_json());
    } else {
        say!(cx, "{event}");
    }
    // Each event goes out as it happens, also when piped.
    let _ = cx.out.flush();
}

#[cfg(test)]
#[path = "tests/events_tests.rs"]
mod tests;

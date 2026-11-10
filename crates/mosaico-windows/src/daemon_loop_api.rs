//! The bar API in the daemon: the hub the sessions are served from, the
//! thread that accepts bars, and the loop's side of what they ask for.

use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mosaico_core::action::Origin;
use mosaico_core::api::{ErrorCode, Hub, Limits, LoopRequest, serve_api};
use mosaico_core::bar::BarHost;

use crate::ipc::ApiPipeListener;
use crate::tiling::TilingManager;

use super::daemon_inputs::Inputs;
use super::daemon_loop_commands::carry_out;
use super::daemon_types::DaemonMsg;

/// How long the daemon waits at stop for the bars to be told goodbye.
const GOODBYE: Duration = Duration::from_secs(1);

/// The bar API while the daemon runs.
pub(super) struct ApiServer {
    hub: Arc<Hub>,
    thread: Option<JoinHandle<()>>,
}

impl ApiServer {
    /// Serves the bar API on the pipe `name` on its own thread, with what
    /// the bars ask of the loop sent through `tx`. The daemon runs without
    /// it when the pipe cannot be created, as when another program holds
    /// its name.
    pub(super) fn start(name: &str, tx: mpsc::Sender<DaemonMsg>) -> Self {
        let to_loop = Box::new(move |request| tx.send(DaemonMsg::Api(request)).is_ok());
        let hub = Hub::new(boot_id(), Limits::default(), to_loop);
        let thread = match ApiPipeListener::new(name) {
            Ok(mut listener) => {
                let served = Arc::clone(&hub);
                let spawned = thread::Builder::new()
                    .name("bar-api".to_string())
                    .spawn(move || serve_api(&mut listener, &served));
                spawned.inspect_err(|e| log_off(name, e)).ok()
            }
            Err(e) => {
                log_off(name, &e);
                None
            }
        };
        Self { hub, thread }
    }

    /// The hub the bars are served from.
    pub(super) fn hub(&self) -> Arc<Hub> {
        Arc::clone(&self.hub)
    }

    /// Tells every bar goodbye, waits a moment for them to be told, and
    /// stops accepting.
    pub(super) fn stop(mut self) {
        self.hub.close("stopping");
        if !self.hub.wait_ended(GOODBYE) {
            mosaico_core::log_debug!("bar API: a session did not end in time");
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn log_off(name: &str, error: &dyn std::fmt::Display) {
    mosaico_core::log_warn!("bar API: not serving {name}: {error}");
}

/// Names this start of the daemon, so a bar can tell it reconnected to a
/// daemon that started again.
fn boot_id() -> String {
    let nanos = (SystemTime::now().duration_since(UNIX_EPOCH)).map_or(0, |d| d.as_nanos());
    format!("{:x}{:x}", nanos as u64, std::process::id())
}

/// Carries out what a session asked of the loop: the state published
/// again, or an action run on the monitor it names, answered once the
/// state it led to is published.
pub(super) fn handle(
    request: LoopRequest,
    manager: &mut TilingManager,
    bars: &mut BarHost,
    inputs: &mut Inputs,
    hotkeys_paused: &mut bool,
) {
    match request {
        LoopRequest::Publish => bars.show(manager),
        LoopRequest::Run {
            action,
            monitor,
            reply,
        } => {
            let Some(index) = manager.tiler().monitor_for(monitor.as_ref()) else {
                let named = monitor.map(|m| m.as_str().to_string()).unwrap_or_default();
                return reply.fail(ErrorCode::NoMonitor, format!("no monitor {named}"));
            };
            let origin = Origin::Bar(index);
            match carry_out(action, origin, manager, bars, inputs, hotkeys_paused) {
                Ok(()) => reply.ok(),
                Err(e) => reply.fail(ErrorCode::Failed, e.to_string()),
            }
        }
    }
}

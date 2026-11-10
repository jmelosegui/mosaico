//! A hub serving in-memory bars, with the test as the daemon loop.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde_json::Value;

use crate::StableMonitorId;
use crate::api::{Daemon, Focus, Hub, Limits, LoopRequest, Monitor, State, serve_api};

use super::fake_pipe::{FakeBar, FakeListener};

/// A `hello` for every topic.
pub(crate) const HELLO: &str = r#"{"hello":{"api":[1],"client":"test"}}"#;

pub(crate) struct World {
    pub hub: Arc<Hub>,
    pub listener: FakeListener,
    requests: Receiver<LoopRequest>,
    server: Option<JoinHandle<()>>,
}

impl World {
    pub fn new() -> Self {
        Self::with(Limits::default())
    }

    pub fn with(limits: Limits) -> Self {
        let (to_loop, requests) = mpsc::channel();
        let to_loop = Box::new(move |request| to_loop.send(request).is_ok());
        let hub = Hub::new("b00t".to_string(), limits, to_loop);
        let listener = FakeListener::new();
        let (mut serving, served) = (listener.clone(), Arc::clone(&hub));
        let server = thread::spawn(move || serve_api(&mut serving, &served));
        Self {
            hub,
            listener,
            requests,
            server: Some(server),
        }
    }

    /// The next request for the loop.
    pub fn request(&self) -> LoopRequest {
        let waited = self.requests.recv_timeout(Duration::from_secs(2));
        waited.expect("a request for the loop")
    }

    /// Whether the loop is asked nothing for a while.
    pub fn quiet(&self) -> bool {
        self.requests
            .recv_timeout(Duration::from_millis(150))
            .is_err()
    }

    /// A bar with a session opened by `hello`, with its welcome and first
    /// snapshot. The state must be published already.
    pub fn open(&self, hello: &str) -> (FakeBar, Value, Value) {
        let mut bar = self.listener.connect();
        bar.send(hello);
        let welcome = bar.recv().expect("welcome");
        let snapshot = bar.recv().expect("snapshot");
        (bar, welcome, snapshot)
    }

    /// Waits up to two seconds for `sessions` to be open.
    pub fn sessions_become(&self, sessions: usize) -> bool {
        for _ in 0..200 {
            if self.hub.sessions() == sessions {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        false
    }
}

impl Drop for World {
    fn drop(&mut self) {
        self.hub.close("test over");
        if let Some(server) = self.server.take() {
            let _ = server.join();
        }
    }
}

/// A state with one monitor showing `workspace`.
pub(crate) fn state(workspace: u32) -> State {
    let monitor = StableMonitorId::new("m-1");
    State {
        daemon: Daemon {
            version: "1.2.3".to_string(),
            api: 1,
            provider: "external".to_string(),
            ..Daemon::default()
        },
        monitors: vec![Monitor {
            id: monitor.clone(),
            active_workspace: workspace,
            ..Monitor::default()
        }],
        focus: Focus {
            monitor: Some(monitor),
            workspace: Some(workspace),
            window: None,
        },
        ..State::default()
    }
}

/// The keys of `value`'s object `key`, in order.
pub(crate) fn keys(value: &Value, key: &str) -> Vec<String> {
    let object = value[key].as_object().expect("an object");
    object.keys().cloned().collect()
}

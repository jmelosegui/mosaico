//! The bar API client over a real pipe, against a daemon's sessions;
//! nothing on the desktop is touched.

use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use mosaico_core::api::client::{ApiClient, Outcome, Query, query};
use mosaico_core::api::{Hub, Limits, State, serve_api};
use mosaico_core::ipc::IpcError;
use serde_json::Value;

use super::{ApiPipeClient, ApiPipeListener};

#[test]
fn a_client_reaches_the_daemon_over_the_pipe_and_reads_the_state() {
    // Arrange
    let name = format!(r"\\.\pipe\mosaico-test-api-client-{}", std::process::id());
    let (to_loop, _requests) = mpsc::channel();
    let hub = Hub::new(
        "b".into(),
        Limits::default(),
        Box::new(move |r| to_loop.send(r).is_ok()),
    );
    let mut state = State::default();
    state.daemon.version = "9.9.9".to_string();
    hub.publish(&state);
    let mut listener = ApiPipeListener::new(&name).unwrap();
    let served = Arc::clone(&hub);
    let server = thread::spawn(move || serve_api(&mut listener, &served));
    let client = ApiPipeClient::new(&name);

    // Act
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut printed = Vec::new();
    let outcome = loop {
        match query(&client, "test", &Query::default(), &mut printed) {
            Outcome::NotRunning if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(10))
            }
            outcome => break outcome,
        }
    };
    hub.close("stopping");
    server.join().unwrap();

    // Assert
    assert!(matches!(outcome, Outcome::Done), "{outcome:?}");
    let printed: Value = serde_json::from_slice(&printed).unwrap();
    assert_eq!(printed["daemon"]["version"], "9.9.9");
}

#[test]
fn a_client_finds_no_daemon_where_no_pipe_is() {
    // Arrange
    let client = ApiPipeClient::new(r"\\.\pipe\mosaico-test-api-nobody-here");

    // Act
    let connected = client.connect();

    // Assert
    assert!(matches!(connected, Err(IpcError::NotRunning)));
}

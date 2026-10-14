use std::io::BufReader;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use mosaico_core::ipc::{Command, Request, Response, read_message, serve, write_message};

use super::*;

/// A pipe name no daemon uses, unique to this test.
fn test_pipe(test: &str) -> String {
    format!(r"\\.\pipe\mosaico-test-{test}-{}", std::process::id())
}

#[test]
fn the_default_instance_keeps_the_usual_pipe() {
    // Arrange
    let instance = Instance::new(None, "cfg", "data").unwrap();

    // Act / Assert
    assert_eq!(pipe_name(&instance), r"\\.\pipe\mosaico");
}

#[test]
fn a_named_instance_gets_its_own_pipe() {
    // Arrange
    let instance = Instance::new(Some("test-7"), "cfg", "data").unwrap();

    // Act / Assert
    assert_eq!(pipe_name(&instance), r"\\.\pipe\mosaico-test-7");
}

#[test]
fn a_missing_pipe_is_reported_as_not_running() {
    // Arrange
    let client = PipeClient::new(&test_pipe("missing"));

    // Act / Assert
    assert!(!client.is_daemon_running());
    assert!(matches!(client.connect(), Err(IpcError::NotRunning)));
}

#[test]
fn a_daemon_busy_with_one_client_still_answers_the_next() {
    // #73
    // Arrange -- a server whose handler holds the first request until
    // the test lets it go, like a daemon in the middle of a slow reply.
    let name = test_pipe("busy");
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let server_name = name.clone();
    let server = thread::spawn(move || {
        let mut server = PipeServer::new(&server_name);
        let mut first = true;
        serve(&mut server, |command| {
            if first {
                first = false;
                let _ = entered_tx.send(());
                let _ = release_rx.recv_timeout(Duration::from_secs(10));
            }
            Some(Response::ok_with_message(format!("{command:?}")))
        })
    });

    // Wait for the pipe to exist, then send the request that is held.
    let client = PipeClient::new(&name);
    for _ in 0..100 {
        if client.is_daemon_running() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let first_name = name.clone();
    let first = thread::spawn(move || PipeClient::new(&first_name).send(&Command::Status));
    entered_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("the server never received the first request");

    // Act -- while the first reply is still in flight.
    let running = client.is_daemon_running();
    let second = client.connect();
    release_tx.send(()).unwrap();

    // Assert
    assert!(running, "a busy daemon must not look missing");
    let mut second = second.expect("the next client must find a free pipe instance");
    write_message(&mut second, &Request::new(Command::Inspect)).unwrap();
    let reply: Response = read_message(&mut BufReader::new(&mut second)).unwrap();
    assert_eq!(reply.message.as_deref(), Some("Inspect"));
    let first = first
        .join()
        .unwrap()
        .expect("the first client gets its reply");
    assert_eq!(first.message.as_deref(), Some("Status"));

    // Cleanup -- stop the server.
    client.send(&Command::Stop).unwrap();
    server.join().unwrap();
}

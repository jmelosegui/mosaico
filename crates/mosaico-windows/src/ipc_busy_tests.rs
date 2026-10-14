use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use mosaico_core::ipc::{Command, IpcClient, IpcError, IpcServer};

use super::*;

#[test]
fn a_daemon_that_never_frees_a_pipe_instance_is_reported_as_not_responding() {
    // #86
    // Arrange -- the daemon took one client and is stuck on it, like a
    // daemon whose process is suspended, and a second client is holding
    // the only free instance. Nothing will free one up.
    let name = format!(r"\\.\pipe\mosaico-test-allbusy-{}", std::process::id());
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let server_name = name.clone();
    let server = thread::spawn(move || {
        let mut server = PipeServer::new(&server_name);
        let first = server.accept();
        let _ = release_rx.recv_timeout(Duration::from_secs(10));
        drop(first);
    });
    let client = PipeClient::new(&name);
    for _ in 0..100 {
        if client.is_daemon_running() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let _first = client.connect().expect("the first client connects");
    thread::sleep(Duration::from_millis(100));
    let _second = client
        .connect()
        .expect("the second client takes the spare instance");

    // Act
    let third = client.send(&Command::Status);

    // Assert
    assert!(
        matches!(third, Err(IpcError::TimedOut)),
        "a daemon that keeps every instance busy should read as not responding, got {third:?}"
    );

    // Cleanup
    release_tx.send(()).unwrap();
    server.join().unwrap();
}

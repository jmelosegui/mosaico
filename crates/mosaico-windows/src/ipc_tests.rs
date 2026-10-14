use std::io::BufReader;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use mosaico_core::ipc::{
    Command, Request, Response, read_message, serve, serve_with_timeout, write_message,
};

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

#[test]
fn a_client_that_leaves_before_it_is_served_does_not_stop_the_server() {
    // #72
    // Arrange -- the server is busy with a first request when a second
    // client connects and closes its end before the server gets to it,
    // as a CLI killed right after it connected would.
    let name = test_pipe("dropout");
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
    drop(client.connect().expect("the second client connects"));
    release_tx.send(()).unwrap();
    first
        .join()
        .unwrap()
        .expect("the first client gets its reply");

    // Act -- the next client after the one that left.
    let reply = client.send(&Command::Inspect);

    // Assert -- the server is still listening and answers it.
    let reply = reply.expect("the server must keep listening after a failed connection");
    assert_eq!(reply.message.as_deref(), Some("Inspect"));

    // Cleanup
    client.send(&Command::Stop).unwrap();
    server.join().unwrap();
}

#[test]
fn a_client_that_never_sends_does_not_block_the_next_one() {
    // #83
    // Arrange -- a server that gives a client 200 ms to send its request,
    // and a client that connects and never sends anything.
    let name = test_pipe("silent");
    let server_name = name.clone();
    let server = thread::spawn(move || {
        let mut server = PipeServer::new(&server_name);
        serve_with_timeout(&mut server, Duration::from_millis(200), |command| {
            Some(Response::ok_with_message(format!("{command:?}")))
        })
    });
    let client = PipeClient::new(&name);
    for _ in 0..100 {
        if client.is_daemon_running() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let silent = client.connect().expect("the silent client connects");

    // Act -- the next client asks for the status while the silent one
    // is still connected.
    let (reply_tx, reply_rx) = mpsc::channel();
    let next_name = name.clone();
    thread::spawn(move || {
        let _ = reply_tx.send(PipeClient::new(&next_name).send(&Command::Status));
    });

    // Assert
    let reply = reply_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("the next client was never answered while a silent client held the daemon")
        .expect("the next client gets a reply");
    assert_eq!(reply.message.as_deref(), Some("Status"));

    // Cleanup
    drop(silent);
    client.send(&Command::Stop).unwrap();
    server.join().unwrap();
}

#[test]
fn a_client_that_never_reads_its_reply_does_not_block_the_next_one() {
    // #85
    // Arrange -- a server that gives each step 200 ms, and a client that
    // sends a request and never reads the reply.
    let name = test_pipe("unread");
    let server_name = name.clone();
    let server = thread::spawn(move || {
        let mut server = PipeServer::new(&server_name);
        serve_with_timeout(&mut server, Duration::from_millis(200), |command| {
            Some(Response::ok_with_message(format!("{command:?}")))
        })
    });
    let client = PipeClient::new(&name);
    for _ in 0..100 {
        if client.is_daemon_running() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let mut unread = client.connect().expect("the first client connects");
    write_message(&mut unread, &Request::new(Command::Inspect)).unwrap();

    // Act -- the next client asks for the status while the first one
    // still has not read its reply.
    let (reply_tx, reply_rx) = mpsc::channel();
    let next_name = name.clone();
    thread::spawn(move || {
        let _ = reply_tx.send(PipeClient::new(&next_name).send(&Command::Status));
    });

    // Assert
    let reply = reply_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("the next client was never answered while the first one did not read its reply")
        .expect("the next client gets a reply");
    assert_eq!(reply.message.as_deref(), Some("Status"));

    // Cleanup
    drop(unread);
    client.send(&Command::Stop).unwrap();
    server.join().unwrap();
}

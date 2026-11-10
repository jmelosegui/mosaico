//! The bar API pipe with real pipe instances and a client of its own;
//! nothing on the desktop is touched.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use mosaico_core::api::{
    ApiConnection, ApiListener, ApiWriter, Hub, Limits, LoopRequest, State, serve_api,
};
use serde_json::Value;

use super::ApiPipeListener;

fn unique(test: &str) -> String {
    format!(r"\\.\pipe\mosaico-test-api-{test}-{}", std::process::id())
}

/// Opens `name` as a bar would, once the daemon has created it.
fn open(name: &str) -> File {
    for _ in 0..200 {
        if let Ok(file) = OpenOptions::new().read(true).write(true).open(name) {
            return file;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("{name} was never created");
}

fn line(reader: &mut impl BufRead) -> Option<Value> {
    let mut text = String::new();
    match reader.read_line(&mut text) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(serde_json::from_str(&text).unwrap()),
    }
}

#[test]
fn a_bar_is_served_on_the_pipe_and_reads_goodbye_after_the_daemon_closes_it() {
    // Arrange
    let name = unique("session");
    let (to_loop, requests) = mpsc::channel();
    let hub = Hub::new(
        "b".into(),
        Limits::default(),
        Box::new(move |r| to_loop.send(r).is_ok()),
    );
    let mut listener = ApiPipeListener::new(&name).unwrap();
    let served = std::sync::Arc::clone(&hub);
    let server = thread::spawn(move || serve_api(&mut listener, &served));
    let mut bar = open(&name);
    bar.write_all(b"{\"hello\":{\"api\":[1]}}\n").unwrap();
    let request = requests.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(matches!(request, LoopRequest::Publish));
    hub.publish(&State::default());
    let mut reader = BufReader::new(bar);
    let welcome = line(&mut reader).unwrap();
    let snapshot = line(&mut reader).unwrap();

    // Act
    hub.close("stopping");
    let ended = hub.wait_ended(Duration::from_secs(2));
    server.join().unwrap();

    // Assert
    assert_eq!(welcome["welcome"]["api"], 1);
    assert_eq!(snapshot["seq"], 1);
    assert!(ended);
    let bye = line(&mut reader);
    assert_eq!(
        bye.map(|b| b["bye"]["reason"].clone()),
        Some("stopping".into())
    );
    assert_eq!(line(&mut reader), None);
}

#[test]
fn a_listener_waiting_for_a_bar_stops_when_told() {
    // Arrange
    let mut listener = ApiPipeListener::new(&unique("stop")).unwrap();
    let stop = listener.stopper();
    let (done, accepted) = mpsc::channel();
    thread::spawn(move || {
        let _ = done.send(listener.accept().map(|c| c.is_none()));
    });
    thread::sleep(Duration::from_millis(50));

    // Act
    stop();

    // Assert
    let stopped = accepted.recv_timeout(Duration::from_secs(2));
    assert!(matches!(stopped, Ok(Ok(true))), "{stopped:?}");
}

#[test]
fn a_write_to_a_bar_that_does_not_read_gives_up_at_the_deadline() {
    // Arrange
    let name = unique("deadline");
    let mut listener = ApiPipeListener::new(&name).unwrap();
    let opener = name.clone();
    let bar = thread::spawn(move || open(&opener));
    let connection = listener.accept().unwrap().unwrap();
    let _bar = bar.join().unwrap();
    let (_reader, mut writer) = connection.split().unwrap();
    writer
        .set_write_timeout(Duration::from_millis(200))
        .unwrap();

    // Act
    let started = Instant::now();
    let chunk = vec![b'x'; 64 * 1024];
    let failed = (0..64).find_map(|_| writer.write_all(&chunk).err());

    // Assert
    let error = failed.expect("the pipe filled up");
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn a_read_waiting_on_one_half_returns_when_the_other_shuts_down() {
    // Arrange
    let name = unique("shutdown");
    let mut listener = ApiPipeListener::new(&name).unwrap();
    let opener = name.clone();
    let bar = thread::spawn(move || open(&opener));
    let connection = listener.accept().unwrap().unwrap();
    let _bar = bar.join().unwrap();
    let (mut reader, mut writer) = connection.split().unwrap();
    let (done, read) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = [0; 16];
        let _ = done.send(std::io::Read::read(&mut reader, &mut buf).ok());
    });
    thread::sleep(Duration::from_millis(50));

    // Act
    writer.shutdown();

    // Assert
    assert_eq!(read.recv_timeout(Duration::from_secs(2)), Ok(Some(0)));
}

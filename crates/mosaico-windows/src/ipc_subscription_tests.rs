//! A subscription over a real named pipe.

use std::thread;
use std::time::Duration;

use mosaico_core::desktop_events::DesktopEvent;
use mosaico_core::ipc::{
    Command, DesktopSnapshot, IpcClient, IpcError, REQUEST_TIMEOUT, Response, Subscribed,
    Subscribers, serve_with_subscriptions, subscribe,
};

use crate::ipc::{PipeClient, PipeServer};

fn empty_desktop() -> DesktopSnapshot {
    DesktopSnapshot {
        monitors: Vec::new(),
        disconnected: Vec::new(),
        displaced_windows: 0,
        daemon_elevated: false,
        focused_monitor: 0,
        focused_window: None,
    }
}

#[test]
fn a_subscriber_hears_events_while_other_commands_are_answered() {
    // Arrange -- a daemon that accepts subscriptions and stops on Stop.
    let name = format!(r"\\.\pipe\mosaico-test-subscribe-{}", std::process::id());
    let subscribers = Subscribers::default();
    let daemon_subscribers = subscribers.clone();
    let server_name = name.clone();
    let server = thread::spawn(move || {
        let mut server = PipeServer::new(&server_name);
        let opened = daemon_subscribers.clone();
        serve_with_subscriptions(
            &mut server,
            REQUEST_TIMEOUT,
            move |command| match command {
                Command::Subscribe { events } => {
                    opened.open(events);
                    Some(Response::ok().with_data(&empty_desktop()))
                }
                _ => Some(Response::ok_with_message("answered")),
            },
            |connection| daemon_subscribers.attach(connection),
        )
    });
    let client = PipeClient::new(&name);
    for _ in 0..200 {
        if client.is_daemon_running() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    // Act
    let mut subscription = match subscribe(&client, Vec::new()) {
        Ok(Subscribed::Accepted(subscription)) => subscription,
        Ok(Subscribed::Refused(r)) => panic!("refused: {r:?}"),
        Err(e) => panic!("failed: {e:?}"),
    };
    let status = client.send(&Command::Status).unwrap();
    subscribers.publish(&[DesktopEvent::ConfigReloaded]);
    let heard = subscription.events.next_event().unwrap();
    client.send(&Command::Stop).unwrap();
    server.join().unwrap();
    subscribers.close();
    let after_stop = subscription.events.next_event();

    // Assert
    assert_eq!(status.message.as_deref(), Some("answered"));
    assert_eq!(heard, DesktopEvent::ConfigReloaded);
    assert!(
        matches!(after_stop, Err(IpcError::Closed | IpcError::Io(_))),
        "{after_stop:?}"
    );
}

//! The real world a script runs in: the daemon over IPC, the system
//! clock and programs started with the standard library.

use std::io::Read;
use std::process::{Command as Program, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::thread;
use std::time::{Duration, Instant};

use mosaico_core::desktop_events::DesktopEvent;
use mosaico_core::ipc::{
    Command, DesktopSnapshot, IpcClient, IpcError, Response, Subscribed, subscribe,
};

use crate::env::{Clock, Daemon, Inbox, Next, Ran, Runner};

/// The daemon behind an IPC client.
pub struct IpcDaemon<C> {
    client: C,
}

impl<C> IpcDaemon<C> {
    /// The daemon `client` reaches.
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C> Daemon for IpcDaemon<C>
where
    C: IpcClient,
    C::Stream: Send + 'static,
{
    fn send(&self, command: &Command) -> Result<Response, IpcError> {
        self.client.send(command)
    }

    fn subscribe(&self) -> Result<(DesktopSnapshot, Box<dyn Inbox>), String> {
        let subscription = match subscribe(&self.client, Vec::new()) {
            Ok(Subscribed::Accepted(subscription)) => subscription,
            Ok(Subscribed::Refused(response)) => {
                return Err(response.message.unwrap_or_else(|| "refused".into()));
            }
            Err(e) => return Err(e.to_string()),
        };
        let (tx, rx) = channel();
        let mut events = subscription.events;
        // Reading waits for the daemon, so it runs on a thread of its own
        // and the script can wait for its timers at the same time.
        thread::Builder::new()
            .name("mosaico-script-events".into())
            .spawn(move || {
                while let Ok(event) = events.next_event() {
                    if tx.send(event).is_err() {
                        return;
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok((subscription.snapshot, Box::new(ChannelInbox(rx))))
    }
}

/// Events a reader thread hands over.
struct ChannelInbox(Receiver<DesktopEvent>);

impl Inbox for ChannelInbox {
    fn next(&mut self, deadline: Option<Instant>) -> Next {
        let Some(deadline) = deadline else {
            return self.0.recv().map_or(Next::Closed, Next::Event);
        };
        let left = deadline.saturating_duration_since(Instant::now());
        match self.0.recv_timeout(left) {
            Ok(event) => Next::Event(event),
            Err(RecvTimeoutError::Timeout) => Next::TimedOut,
            Err(RecvTimeoutError::Disconnected) => Next::Closed,
        }
    }
}

/// The system clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep_until(&self, deadline: Instant) {
        thread::sleep(deadline.saturating_duration_since(Instant::now()));
    }
}

/// Runs programs with the standard library, reading what they write.
pub struct ProcessRunner;

/// How often a program with a timeout is checked on.
const POLL: Duration = Duration::from_millis(10);

impl Runner for ProcessRunner {
    fn run(&self, argv: &[String], timeout: Option<Duration>) -> Result<Ran, String> {
        let (program, args) = argv.split_first().ok_or("no program to run")?;
        let mut child = Program::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("could not run {program}: {e}"))?;
        // Read both streams while the program runs, so a full pipe never
        // stops it.
        let stdout = child.stdout.take().map(read_all);
        let stderr = child.stderr.take().map(read_all);
        let deadline = timeout.map(|t| Instant::now() + t);
        let mut timed_out = false;
        let status = loop {
            match child.try_wait().map_err(|e| e.to_string())? {
                Some(status) => break status,
                None if deadline.is_some_and(|d| Instant::now() >= d) => {
                    let _ = child.kill();
                    timed_out = true;
                    break child.wait().map_err(|e| e.to_string())?;
                }
                None => thread::sleep(POLL),
            }
        };
        let text = |reader: Option<thread::JoinHandle<String>>| {
            reader.and_then(|r| r.join().ok()).unwrap_or_default()
        };
        Ok(Ran {
            code: if timed_out { None } else { status.code() },
            stdout: text(stdout),
            stderr: text(stderr),
            timed_out,
        })
    }
}

fn read_all(mut stream: impl Read + Send + 'static) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stream.read_to_end(&mut bytes);
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

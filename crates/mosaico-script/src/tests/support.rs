//! A fake daemon, clock and programs to run scripts against.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Duration, Instant};

use mosaico_core::desktop_events::DesktopEvent;
use mosaico_core::ipc::{
    Command, DesktopSnapshot, IpcError, MonitorSnapshot, Response, WindowSnapshot,
    WorkspaceSnapshot,
};
use mosaico_core::monitor::MonitorKey;
use mosaico_core::system::Launcher;
use mosaico_core::{MonitorId, Rect, WindowId};

use crate::{Clock, Daemon, Env, Inbox, Next, Options, Outcome, Ran, Runner, Script};

pub struct FakeClock(Cell<Instant>);

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        self.0.get()
    }

    fn sleep_until(&self, deadline: Instant) {
        if deadline > self.0.get() {
            self.0.set(deadline);
        }
    }
}

/// Hands out its events at once, then lets the clock run to the deadline,
/// or ends without one.
struct FakeInbox {
    events: VecDeque<DesktopEvent>,
    clock: Rc<FakeClock>,
}

impl Inbox for FakeInbox {
    fn next(&mut self, deadline: Option<Instant>) -> Next {
        if let Some(event) = self.events.pop_front() {
            return Next::Event(event);
        }
        match deadline {
            Some(deadline) => {
                self.clock.sleep_until(deadline);
                Next::TimedOut
            }
            None => Next::Closed,
        }
    }
}

pub struct FakeDaemon {
    pub running: Cell<bool>,
    pub desktop: RefCell<DesktopSnapshot>,
    pub sent: RefCell<Vec<Command>>,
    /// Replies to commands other than `State`, in order; ok without one.
    pub replies: RefCell<VecDeque<Response>>,
    /// The events of each subscription, in order; none after the last.
    pub subscriptions: RefCell<VecDeque<Vec<DesktopEvent>>>,
    pub subscribed: Cell<u32>,
    clock: Rc<FakeClock>,
}

impl Daemon for FakeDaemon {
    fn send(&self, command: &Command) -> Result<Response, IpcError> {
        if !self.running.get() {
            return Err(IpcError::NotRunning);
        }
        self.sent.borrow_mut().push(command.clone());
        if *command == Command::State {
            return Ok(Response::ok().with_data(&*self.desktop.borrow()));
        }
        Ok(self
            .replies
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(Response::ok))
    }

    fn subscribe(&self) -> Result<(DesktopSnapshot, Box<dyn Inbox>), String> {
        if !self.running.get() {
            return Err("mosaico is not running".into());
        }
        self.subscribed.set(self.subscribed.get() + 1);
        let events = self
            .subscriptions
            .borrow_mut()
            .pop_front()
            .unwrap_or_default();
        let inbox = FakeInbox {
            events: events.into(),
            clock: self.clock.clone(),
        };
        Ok((self.desktop.borrow().clone(), Box::new(inbox)))
    }
}

#[derive(Default)]
pub struct FakeLauncher(pub RefCell<Vec<Vec<String>>>);

impl Launcher for FakeLauncher {
    fn launch(&self, command: &str, args: &[String]) -> mosaico_core::Result<()> {
        let mut line = vec![command.to_string()];
        line.extend(args.iter().cloned());
        self.0.borrow_mut().push(line);
        Ok(())
    }
}

#[derive(Default)]
pub struct FakeRunner(pub RefCell<Vec<(Vec<String>, Option<Duration>)>>);

impl Runner for FakeRunner {
    fn run(&self, argv: &[String], timeout: Option<Duration>) -> Result<Ran, String> {
        self.0.borrow_mut().push((argv.to_vec(), timeout));
        Ok(Ran {
            code: Some(0),
            stdout: format!("ran {}\n", argv.join(" ")),
            stderr: String::new(),
            timed_out: false,
        })
    }
}

/// Everything a script ran against, to look at afterwards.
pub struct World {
    pub daemon: Rc<FakeDaemon>,
    pub clock: Rc<FakeClock>,
    pub start: Instant,
    pub launcher: Rc<FakeLauncher>,
    pub runner: Rc<FakeRunner>,
    pub out: Rc<RefCell<Vec<u8>>>,
    pub err: Rc<RefCell<Vec<u8>>>,
}

impl World {
    /// A running daemon with [`desktop`].
    pub fn new() -> Self {
        let start = Instant::now();
        let clock = Rc::new(FakeClock(Cell::new(start)));
        let daemon = Rc::new(FakeDaemon {
            running: Cell::new(true),
            desktop: RefCell::new(desktop()),
            sent: RefCell::default(),
            replies: RefCell::default(),
            subscriptions: RefCell::default(),
            subscribed: Cell::new(0),
            clock: clock.clone(),
        });
        Self {
            daemon,
            clock,
            start,
            launcher: Rc::default(),
            runner: Rc::default(),
            out: Rc::default(),
            err: Rc::default(),
        }
    }

    pub fn env(&self) -> Env {
        Env {
            daemon: self.daemon.clone(),
            clock: self.clock.clone(),
            launcher: self.launcher.clone(),
            runner: self.runner.clone(),
            out: self.out.clone(),
            err: self.err.clone(),
        }
    }

    /// The events the next subscription gets.
    pub fn events(&self, events: Vec<DesktopEvent>) {
        self.daemon.subscriptions.borrow_mut().push_back(events);
    }

    /// How long the fake clock ran.
    pub fn elapsed(&self) -> Duration {
        self.clock.now() - self.start
    }

    pub fn script(&self, source: &str) -> Script {
        self.script_with(source, Options::default())
    }

    pub fn script_with(&self, source: &str, options: Options) -> Script {
        let options = Options {
            name: "test".into(),
            ..options
        };
        Script::new(source.into(), self.env(), options).expect("the script loads")
    }

    pub fn run(&self, source: &str) -> Outcome {
        self.script(source).run()
    }

    pub fn out(&self) -> String {
        String::from_utf8(self.out.borrow().clone()).unwrap()
    }

    pub fn err(&self) -> String {
        String::from_utf8(self.err.borrow().clone()).unwrap()
    }

    /// The commands sent, without the `State` reads.
    pub fn sent(&self) -> Vec<Command> {
        self.daemon
            .sent
            .borrow()
            .iter()
            .filter(|c| **c != Command::State)
            .cloned()
            .collect()
    }
}

pub fn wid(raw: u64) -> WindowId {
    WindowId::from_raw(raw)
}

pub fn mid(raw: u64) -> MonitorId {
    MonitorId::from_raw(raw)
}

fn window(raw: u64, class: &str, title: &str) -> WindowSnapshot {
    WindowSnapshot {
        id: wid(raw),
        visible: true,
        minimized: false,
        cloaked: false,
        elevated: false,
        rect: None,
        class: class.into(),
        title: title.into(),
    }
}

/// Monitor 10 on its second workspace: Notepad (1) and Code (2) on its
/// first, Terminal (3) floating on its second; monitor 20 empty. Code
/// has focus.
pub fn desktop() -> DesktopSnapshot {
    let mut first = vec![WorkspaceSnapshot::default(); 3];
    first[0].windows = vec![
        window(1, "Notepad", "a.txt - Notepad"),
        window(2, "Code", "main.rs"),
    ];
    first[1].floating = vec![window(3, "Terminal", "pwsh")];
    DesktopSnapshot {
        monitors: vec![
            MonitorSnapshot {
                id: mid(10),
                key: MonitorKey::default(),
                work_area: Rect::new(0, 0, 1920, 1080),
                active_workspace: 1,
                workspaces: first,
            },
            MonitorSnapshot {
                id: mid(20),
                key: MonitorKey::default(),
                work_area: Rect::new(1920, 0, 1920, 1080),
                active_workspace: 0,
                workspaces: vec![WorkspaceSnapshot::default(); 3],
            },
        ],
        disconnected: Vec::new(),
        displaced_windows: 0,
        daemon_elevated: false,
        focused_monitor: 0,
        focused_window: Some(wid(2)),
    }
}

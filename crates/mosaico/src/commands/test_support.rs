//! Fakes for testing commands without a daemon, processes, registry or
//! network.
//!
//! [`World`] stands in for everything outside the CLI at once (the
//! daemon, the process table, the autostart registration, program files,
//! the desktop, GitHub and the clock) so a test can describe a
//! situation, such as "the daemon stopped answering but its process is
//! still alive", in a few fields.

#[path = "test_support_autostart.rs"]
mod autostart_fake;
mod desktop;
mod harness;
#[path = "test_support_release.rs"]
mod release;
mod windows;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use mosaico_core::ipc::{Command, IpcError, Response};
use mosaico_core::system::{LaunchCommand, ProcessControl};
use mosaico_core::{Error, Rect, Result, WindowId};

use super::context::{Clock, Daemon};

pub use autostart_fake::{EXE, MOVED_AWAY};
pub use harness::Harness;
pub use release::{ASSET, BINARY, FakeInstaller, Releases};
pub use windows::Frames;

/// How long a fake process stays alive.
#[derive(Debug, Clone, Copy)]
pub enum Lifetime {
    /// Until it is killed.
    Forever,
    /// For this many more liveness checks.
    Checks(u32),
}

/// What launching a daemon does in the fake world.
#[derive(Debug, Clone)]
pub enum Launch {
    /// A daemon with this pid starts and answers.
    Answers(u32),
    /// A process with this pid starts and exits without answering, as a
    /// daemon rejected by the single instance lock does.
    ExitsAtOnce(u32),
    /// A process with this pid starts and keeps running, but never
    /// answers.
    Silent(u32),
    /// The process cannot be started, failing with this message.
    Fails(String),
}

/// The outside world, faked.
pub struct World {
    /// Whether a daemon answers.
    pub daemon_running: Cell<bool>,
    /// Whether the liveness check misses a running daemon, as it can
    /// while the daemon is busy with another client.
    pub looks_missing: Cell<bool>,
    /// Replies the daemon gives, in order. Without one it replies ok.
    pub replies: RefCell<VecDeque<Result<Response, IpcError>>>,
    /// Commands the daemon received.
    pub sent: RefCell<Vec<Command>>,
    /// Running processes.
    pub processes: RefCell<HashMap<u32, Lifetime>>,
    /// Whether killing a process works.
    pub can_kill: Cell<bool>,
    /// What `launch_daemon` does.
    pub launch: RefCell<Launch>,
    /// What is registered to start at login.
    pub autostart: RefCell<Option<LaunchCommand>>,
    /// Whether the user turned the registration off in the system's
    /// startup settings.
    pub turned_off: Cell<bool>,
    /// Error the autostart registration fails with, if any.
    pub autostart_error: RefCell<Option<String>>,
    /// Windows placed, and where, in order.
    pub placed: RefCell<Vec<(WindowId, Rect)>>,
    /// Error placing a window fails with, if any.
    pub place_error: RefCell<Option<String>>,
    /// What asking the desktop to move windows frame by frame does.
    pub frames: Cell<Frames>,
    /// Whether the system's settings let windows be animated.
    pub animations_allowed: Cell<bool>,
    /// Program files that exist.
    pub files: RefCell<Vec<PathBuf>>,
    /// Whether the executable commands run from cannot be found.
    pub exe_missing: Cell<bool>,
    /// The releases GitHub has published.
    pub releases: Releases,
    /// How releases are installed.
    pub installer: FakeInstaller,
    /// The fake time.
    pub now: Cell<Instant>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            daemon_running: Cell::new(false),
            looks_missing: Cell::new(false),
            replies: RefCell::default(),
            sent: RefCell::default(),
            processes: RefCell::default(),
            can_kill: Cell::new(true),
            launch: RefCell::new(Launch::Answers(4321)),
            autostart: RefCell::default(),
            turned_off: Cell::new(false),
            autostart_error: RefCell::default(),
            placed: RefCell::default(),
            place_error: RefCell::default(),
            frames: Cell::new(Frames::Unsupported),
            animations_allowed: Cell::new(true),
            files: RefCell::default(),
            exe_missing: Cell::new(false),
            releases: Releases::default(),
            installer: FakeInstaller::default(),
            now: Cell::new(Instant::now()),
        }
    }
}

impl World {
    /// A daemon process `pid` that answers on its channel.
    pub fn daemon_up(&self, pid: u32) {
        self.daemon_running.set(true);
        self.processes.borrow_mut().insert(pid, Lifetime::Forever);
    }

    /// Queues the daemon's next reply.
    pub fn reply(&self, reply: Result<Response, IpcError>) {
        self.replies.borrow_mut().push_back(reply);
    }
}

impl Daemon for World {
    fn is_running(&self) -> bool {
        self.daemon_running.get() && !self.looks_missing.get()
    }

    fn send(&self, command: &Command) -> Result<Response, IpcError> {
        if !self.daemon_running.get() {
            return Err(IpcError::NotRunning);
        }
        self.sent.borrow_mut().push(command.clone());
        let reply = self
            .replies
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| Ok(Response::ok()));
        if *command == Command::Stop && reply.is_ok() {
            // The daemon stops answering as soon as it accepts a stop.
            self.daemon_running.set(false);
        }
        reply
    }
}

impl ProcessControl for World {
    fn launch_daemon(&self) -> Result<u32> {
        match self.launch.borrow().clone() {
            Launch::Answers(pid) => {
                self.daemon_up(pid);
                Ok(pid)
            }
            Launch::ExitsAtOnce(pid) => Ok(pid),
            Launch::Silent(pid) => {
                self.processes.borrow_mut().insert(pid, Lifetime::Forever);
                Ok(pid)
            }
            Launch::Fails(message) => Err(failure(message)),
        }
    }

    fn is_alive(&self, pid: u32) -> bool {
        let mut processes = self.processes.borrow_mut();
        match processes.get(&pid).copied() {
            None => false,
            Some(Lifetime::Forever) => true,
            Some(Lifetime::Checks(0)) => {
                processes.remove(&pid);
                false
            }
            Some(Lifetime::Checks(n)) => {
                processes.insert(pid, Lifetime::Checks(n - 1));
                true
            }
        }
    }

    fn kill(&self, pid: u32) -> Result<()> {
        if !self.can_kill.get() {
            return Err(Error::platform("end the process", "access denied"));
        }
        match self.processes.borrow_mut().remove(&pid) {
            // A process with no checks left exited after the last one.
            // Ending a process that has exited fails, as the contract says.
            None | Some(Lifetime::Checks(0)) => Err(Error::platform(
                "open the process to end it",
                "no such process",
            )),
            Some(_) => Ok(()),
        }
    }
}

/// An error that reads exactly `message`, so a test can pin what the
/// command prints.
fn failure(message: String) -> Error {
    Error::Io(io::Error::other(message))
}

impl Clock for World {
    fn now(&self) -> Instant {
        self.now.get()
    }

    fn sleep(&self, duration: Duration) {
        self.now.set(self.now.get() + duration);
    }
}

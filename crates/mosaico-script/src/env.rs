//! What a script reaches outside itself: the daemon, time, programs and
//! output. Each is passed in, so a script runs against fakes in tests.

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::time::{Duration, Instant};

use mosaico_core::desktop_events::DesktopEvent;
use mosaico_core::ipc::{Command, DesktopSnapshot, IpcError, Response};
use mosaico_core::system::Launcher;

/// The daemon a script talks to.
pub trait Daemon {
    /// Sends one command and waits for its reply.
    fn send(&self, command: &Command) -> Result<Response, IpcError>;

    /// Subscribes to every desktop event. Returns the desktop at that
    /// moment and the events that follow it, or why the daemon could not
    /// be subscribed to.
    fn subscribe(&self) -> Result<(DesktopSnapshot, Box<dyn Inbox>), String>;
}

/// What waiting for the next event found.
#[derive(Debug, Clone, PartialEq)]
pub enum Next {
    /// An event arrived.
    Event(DesktopEvent),
    /// The deadline passed first.
    TimedOut,
    /// The subscription ended, as it does when the daemon stops.
    Closed,
}

/// The events of a subscription.
pub trait Inbox {
    /// Waits for the next event until `deadline`, or as long as it takes
    /// without one.
    fn next(&mut self, deadline: Option<Instant>) -> Next;
}

/// The time a script runs in.
pub trait Clock {
    /// Now.
    fn now(&self) -> Instant;

    /// Waits until `deadline`.
    fn sleep_until(&self, deadline: Instant);
}

/// What a program a script ran did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ran {
    /// Its exit code, when it exited on its own.
    pub code: Option<i32>,
    /// What it wrote to its standard output.
    pub stdout: String,
    /// What it wrote to its standard error.
    pub stderr: String,
    /// Whether it was stopped for running past its timeout.
    pub timed_out: bool,
}

/// Runs programs and waits for them, without a shell between.
pub trait Runner {
    /// Runs `argv[0]` with the rest as its arguments, stopping it after
    /// `timeout` if it has one. Fails only when the program could not be
    /// started; a program that fails is still a [`Ran`].
    fn run(&self, argv: &[String], timeout: Option<Duration>) -> Result<Ran, String>;
}

/// Where a script's output goes.
pub type Output = Rc<RefCell<dyn Write>>;

/// Everything a script may use.
#[derive(Clone)]
pub struct Env {
    /// The daemon.
    pub daemon: Rc<dyn Daemon>,
    /// Time.
    pub clock: Rc<dyn Clock>,
    /// Starts programs the way the user's shell would, detached.
    pub launcher: Rc<dyn Launcher>,
    /// Runs programs and waits for their output.
    pub runner: Rc<dyn Runner>,
    /// `print`.
    pub out: Output,
    /// `mosaico.log` and errors in handlers.
    pub err: Output,
}

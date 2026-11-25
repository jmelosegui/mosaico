//! What scripts run against in command tests: a daemon that hands out
//! the events queued for each subscription, a clock that never waits,
//! and output kept for the test.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use mosaico_core::desktop_events::DesktopEvent;
use mosaico_core::ipc::{Command, DesktopSnapshot, IpcError, Response};
use mosaico_core::system::{LaunchCommand, Launcher, Opener, ScriptAutostart, Visibility};
use mosaico_script::{Clock, Daemon, Env, Inbox, Next, Ran, Runner};

use super::{World, empty_desktop};

/// The daemon as scripts see it.
#[derive(Default)]
pub struct ScriptDaemon {
    /// The events of each subscription, in order. Without one queued,
    /// subscribing fails as with no daemon running.
    pub subscriptions: RefCell<VecDeque<Vec<DesktopEvent>>>,
    /// The commands scripts sent.
    pub sent: RefCell<Vec<Command>>,
}

struct Queued(VecDeque<DesktopEvent>);

impl Inbox for Queued {
    fn next(&mut self, deadline: Option<Instant>) -> Next {
        match (self.0.pop_front(), deadline) {
            (Some(event), _) => Next::Event(event),
            (None, Some(_)) => Next::TimedOut,
            (None, None) => Next::Closed,
        }
    }
}

impl Daemon for ScriptDaemon {
    fn send(&self, command: &Command) -> Result<Response, IpcError> {
        self.sent.borrow_mut().push(command.clone());
        Ok(Response::ok().with_data(&empty_desktop()))
    }

    fn subscribe(&self) -> Result<(DesktopSnapshot, Box<dyn Inbox>), String> {
        let events = self.subscriptions.borrow_mut().pop_front();
        let events = events.ok_or("mosaico is not running")?;
        Ok((empty_desktop(), Box::new(Queued(events.into()))))
    }
}

/// A clock that jumps to where a script waits until.
pub struct ScriptClock(Cell<Instant>);

impl Default for ScriptClock {
    fn default() -> Self {
        Self(Cell::new(Instant::now()))
    }
}

impl Clock for ScriptClock {
    fn now(&self) -> Instant {
        self.0.get()
    }

    fn sleep_until(&self, deadline: Instant) {
        self.0.set(self.0.get().max(deadline));
    }
}

/// Neither starts nor runs anything.
pub struct NoPrograms;

impl Launcher for NoPrograms {
    fn launch(&self, command: &str, _: &[String], _: Visibility) -> mosaico_core::Result<()> {
        Err(mosaico_core::Error::Platform {
            operation: "start a program",
            source: format!("{command} not started in a test").into(),
        })
    }
}

impl Runner for NoPrograms {
    fn run(&self, argv: &[String], _: Option<Duration>) -> Result<Ran, String> {
        Err(format!("{} not run in a test", argv.join(" ")))
    }
}

/// Everything scripts of one test run against.
#[derive(Default, Clone)]
pub struct Scripts {
    pub daemon: Rc<ScriptDaemon>,
    pub clock: Rc<ScriptClock>,
    pub out: Rc<RefCell<Vec<u8>>>,
}

impl Scripts {
    /// The environment of a script.
    pub fn env(&self) -> Env {
        Env {
            daemon: self.daemon.clone(),
            clock: self.clock.clone(),
            launcher: Rc::new(NoPrograms),
            runner: Rc::new(NoPrograms),
            out: self.out.clone(),
            err: self.out.clone(),
        }
    }

    /// What scripts printed and logged.
    pub fn printed(&self) -> String {
        String::from_utf8_lossy(&self.out.borrow()).into_owned()
    }
}

impl World {
    /// What the opener was asked to open, in order.
    pub fn opened(&self) -> Vec<PathBuf> {
        self.opened.borrow().clone()
    }
}

impl Opener for World {
    fn open(&self, path: &Path) -> mosaico_core::Result<()> {
        self.opened.borrow_mut().push(path.to_path_buf());
        Ok(())
    }
}

impl ScriptAutostart for World {
    fn is_enabled(&self, name: &str) -> mosaico_core::Result<bool> {
        Ok(self.script_logins.borrow().contains_key(name))
    }

    fn enable(&self, name: &str, command: &LaunchCommand) -> mosaico_core::Result<()> {
        self.script_logins
            .borrow_mut()
            .insert(name.to_string(), command.clone());
        Ok(())
    }

    fn disable(&self, name: &str) -> mosaico_core::Result<()> {
        self.script_logins.borrow_mut().remove(name);
        Ok(())
    }
}

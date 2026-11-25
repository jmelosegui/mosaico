//! A loaded script and the loop that runs its handlers and timers.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

use mlua::{Function, Lua};

use mosaico_core::desktop_events::{DesktopEvent, EventKind};

use crate::env::{Env, Inbox, Next};
use crate::{api, sandbox};

/// How a script ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// It ran to its end, and has no handlers or timers left.
    Finished,
    /// It called `mosaico.stop` with this exit code.
    Stopped(i32),
    /// It failed: an error while loading or running its file, or in a
    /// handler with `fail_fast`. The message says where.
    Failed(String),
    /// The daemon ended its subscription, as it does when it stops. The
    /// handlers are still there: run it again once the daemon is back.
    DaemonGone,
}

/// A timer a script set.
pub(crate) struct Timer {
    pub(crate) id: u64,
    pub(crate) due: Instant,
    pub(crate) every: Option<Duration>,
    pub(crate) run: Function,
}

/// What the `mosaico` functions share with the loop.
pub(crate) struct Shared {
    pub(crate) env: Env,
    pub(crate) handlers: Vec<(EventKind, Function)>,
    pub(crate) timers: Vec<Timer>,
    pub(crate) next_timer: u64,
    pub(crate) inbox: Option<Box<dyn Inbox>>,
    /// Events that arrived while a script waited for something else.
    pub(crate) pending: VecDeque<DesktopEvent>,
    /// The code `mosaico.stop` asked to exit with.
    pub(crate) stop: Option<i32>,
}

/// Raised by `mosaico.stop` to unwind the script; never shown.
pub(crate) const STOP: &str = "mosaico.stop";

pub(crate) type State = Rc<RefCell<Shared>>;

impl Shared {
    /// Subscribes when not subscribed yet. Says whether it is.
    pub(crate) fn ensure_subscribed(&mut self) -> bool {
        if self.inbox.is_none()
            && let Ok((_, inbox)) = self.env.daemon.subscribe()
        {
            self.inbox = Some(inbox);
        }
        self.inbox.is_some()
    }

    /// Waits for the next event until `deadline`, taking those that
    /// arrived earlier first.
    pub(crate) fn next_event(&mut self, deadline: Option<Instant>) -> Next {
        if let Some(event) = self.pending.pop_front() {
            return Next::Event(event);
        }
        match self.inbox.as_mut() {
            Some(inbox) => {
                let next = inbox.next(deadline);
                if next == Next::Closed {
                    self.inbox = None;
                }
                next
            }
            None => Next::Closed,
        }
    }
}

/// Options for running a script.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// The script's name, used in its error messages.
    pub name: String,
    /// Where `require` finds modules; none when absent.
    pub lib: Option<std::path::PathBuf>,
    /// `key=value` arguments, which the script reads from `mosaico.args`.
    pub args: Vec<(String, String)>,
    /// Stop at the first error in a handler or timer instead of going on.
    pub fail_fast: bool,
}

/// A script, ready to run.
pub struct Script {
    lua: Lua,
    state: State,
    source: String,
    options: Options,
    started: bool,
}

impl Script {
    /// Prepares `source` to run in `env`.
    pub fn new(source: String, env: Env, options: Options) -> Result<Self, String> {
        let lua = Lua::new();
        let state = Rc::new(RefCell::new(Shared {
            env,
            handlers: Vec::new(),
            timers: Vec::new(),
            next_timer: 1,
            inbox: None,
            pending: VecDeque::new(),
            stop: None,
        }));
        sandbox::apply(&lua, options.lib.as_deref()).map_err(|e| e.to_string())?;
        api::install(&lua, &state, &options.args).map_err(|e| e.to_string())?;
        Ok(Self {
            lua,
            state,
            source,
            options,
            started: false,
        })
    }

    /// The directory scripts' modules are found in, as given.
    pub fn lib(&self) -> Option<&Path> {
        self.options.lib.as_deref()
    }

    /// Runs the script's file, the first time, then its handlers and
    /// timers until it stops, has nothing left to do, or the daemon goes
    /// away. After [`Outcome::DaemonGone`], running again subscribes again
    /// and goes on with the same handlers.
    pub fn run(&mut self) -> Outcome {
        if !self.started {
            self.started = true;
            let chunk = self.lua.load(&self.source).set_name(&self.options.name);
            if let Err(e) = chunk.exec() {
                return self.ended_by(&e);
            }
        }
        self.event_loop()
    }

    fn event_loop(&self) -> Outcome {
        loop {
            if let Some(code) = self.state.borrow().stop {
                return Outcome::Stopped(code);
            }
            let (has_handlers, next_due) = {
                let shared = self.state.borrow();
                let due = shared.timers.iter().map(|t| t.due).min();
                (!shared.handlers.is_empty(), due)
            };
            if !has_handlers && next_due.is_none() {
                return Outcome::Finished;
            }
            let now = self.state.borrow().env.clock.now();
            if let Some(due) = next_due
                && due <= now
            {
                if let Some(outcome) = self.fire_timer(now) {
                    return outcome;
                }
                continue;
            }
            if !has_handlers {
                let clock = self.state.borrow().env.clock.clone();
                if let Some(due) = next_due {
                    clock.sleep_until(due);
                }
                continue;
            }
            let next = {
                let mut shared = self.state.borrow_mut();
                if !shared.ensure_subscribed() {
                    return Outcome::DaemonGone;
                }
                shared.next_event(next_due)
            };
            match next {
                Next::Event(event) => {
                    if let Some(outcome) = self.dispatch(&event) {
                        return outcome;
                    }
                }
                Next::TimedOut => {}
                Next::Closed => return Outcome::DaemonGone,
            }
        }
    }

    /// Runs the first timer due at `now`, and sets it again if it repeats.
    fn fire_timer(&self, now: Instant) -> Option<Outcome> {
        let run = {
            let mut shared = self.state.borrow_mut();
            let index = shared.timers.iter().position(|t| t.due <= now)?;
            let timer = &mut shared.timers[index];
            let run = timer.run.clone();
            match timer.every {
                Some(every) => timer.due = now + every,
                None => {
                    shared.timers.remove(index);
                }
            }
            run
        };
        self.handled(run.call::<()>(()))
    }

    /// Calls every handler of the event's kind with it.
    fn dispatch(&self, event: &DesktopEvent) -> Option<Outcome> {
        let kind = event.kind();
        let handlers: Vec<Function> = self
            .state
            .borrow()
            .handlers
            .iter()
            .filter(|(k, _)| *k == kind)
            .map(|(_, f)| f.clone())
            .collect();
        if handlers.is_empty() {
            return None;
        }
        let value = match api::event_value(&self.lua, event) {
            Ok(value) => value,
            Err(e) => return self.handled(Err(e)),
        };
        for handler in handlers {
            if let Some(outcome) = self.handled(handler.call::<()>(value.clone())) {
                return Some(outcome);
            }
        }
        None
    }

    /// What a handler's or timer's result means for the loop: nothing to
    /// do, or how the script ends.
    fn handled(&self, result: mlua::Result<()>) -> Option<Outcome> {
        let e = result.err()?;
        if self.state.borrow().stop.is_some() {
            return Some(self.ended_by(&e));
        }
        if self.options.fail_fast {
            return Some(self.ended_by(&e));
        }
        let err = self.state.borrow().env.err.clone();
        let _ = writeln!(err.borrow_mut(), "Error: {}", message(&e));
        None
    }

    fn ended_by(&self, e: &mlua::Error) -> Outcome {
        match self.state.borrow().stop {
            Some(code) => Outcome::Stopped(code),
            None => Outcome::Failed(message(e)),
        }
    }
}

/// An error's message without the traceback Lua adds to it.
fn message(e: &mlua::Error) -> String {
    match e {
        mlua::Error::CallbackError { cause, .. } => message(cause),
        mlua::Error::RuntimeError(text) | mlua::Error::SyntaxError { message: text, .. } => {
            text.lines().next().unwrap_or_default().to_string()
        }
        other => other
            .to_string()
            .lines()
            .next()
            .unwrap_or_default()
            .to_string(),
    }
}

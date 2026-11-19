#![deny(missing_docs)]

//! Lua scripts that drive Mosaico.
//!
//! A script runs in its own process, as a client of the daemon: it reads
//! the desktop, sends actions and orders, handles desktop events and sets
//! timers through the `mosaico` table. Nothing here runs in the daemon,
//! so a script that fails or never returns only stops itself.
//!
//! Everything a script reaches outside itself is an [`Env`], so scripts
//! are tested against a fake daemon, clock and programs.

mod api;
mod convert;
mod env;
mod live;
mod runtime;
mod sandbox;

pub use env::{Clock, Daemon, Env, Inbox, Next, Output, Ran, Runner};
pub use live::{IpcDaemon, ProcessRunner, SystemClock};
pub use runtime::{Options, Outcome, Script};

#[cfg(test)]
mod tests;

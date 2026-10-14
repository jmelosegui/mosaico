//! What the daemon loop runs, and when.
//!
//! The daemon receives window events, user actions, CLI commands, config
//! reloads and timer ticks on one queue, and drives the tiling manager
//! and the bar from them. [`Scheduler`](crate::daemon::Scheduler) makes
//! every timing decision of that loop: what runs first, which events say
//! nothing new, when a display change has settled and when the effects
//! the tiling manager holds back are flushed. It never reads the clock.
//! The platform loop tells it when it starts waiting, hands it whatever
//! ended the wait, and runs the [`Step`](crate::daemon::Step)s it gets
//! back.

mod dedup;
mod scheduler;
mod settle;

pub use scheduler::{Message, Scheduler, Step, Wake};
pub use settle::SETTLE;

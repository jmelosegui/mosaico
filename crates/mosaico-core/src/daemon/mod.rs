//! What the daemon loop runs, and when.
//!
//! The daemon receives window events, user actions, CLI commands, config
//! reloads, timer ticks and, while windows move, display frames on one
//! queue, and drives the tiling manager and the bar from them.
//! [`Scheduler`](crate::daemon::Scheduler) makes every timing decision of
//! that loop: what runs first, which events say nothing new, when a
//! display change has settled, when the effects the tiling manager holds
//! back are flushed, when windows in motion take their next step and when
//! the wheel's turns on a bar are applied. It never reads the clock. The
//! platform loop tells it when it starts waiting, hands it whatever ended
//! the wait, and runs the [`Step`](crate::daemon::Step)s it gets back. The
//! frames come from the platform's
//! [`FrameClock`](crate::daemon::FrameClock), which can be a
//! [`FramePacer`](crate::daemon::FramePacer) waiting on the display
//! through a [`FrameWait`](crate::daemon::FrameWait) of the platform's.

mod dedup;
mod frame_clock;
mod frame_pacer;
mod frame_wait;
mod messages;
mod scheduler;
mod settle;

pub use frame_clock::FrameClock;
pub use frame_pacer::{FramePacer, FrameToken};
pub use frame_wait::FrameWait;
pub use messages::{Message, Step, Wake};
pub use scheduler::Scheduler;
pub use settle::SETTLE;

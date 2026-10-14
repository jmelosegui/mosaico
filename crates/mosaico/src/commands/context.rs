//! What a command needs from the outside world.
//!
//! Commands take a [`Context`] instead of reaching for the daemon, the
//! process table, the registry, the desktop, the clock or the terminal
//! directly, and return an [`Exit`] instead of ending the process.
//! `main` builds the real context and turns the exit into a status code;
//! the unit tests build one from fakes.

use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use mosaico_core::WindowControl;
use mosaico_core::ipc::{Command, IpcClient, IpcError, Response};
use mosaico_core::pid::PidFile;
use mosaico_core::system::{Autostart, ProcessControl};

/// How a command ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    /// It did what was asked.
    Success,
    /// It could not; the reason was written to the error stream.
    Failure,
}

impl Exit {
    /// The process exit status for this outcome.
    pub fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
        }
    }
}

/// The running daemon, as far as a command is concerned.
///
/// An object-safe view of [`IpcClient`], which every client gets for
/// free, so commands can hold it as `&dyn Daemon`.
pub trait Daemon {
    /// Whether a daemon is listening.
    fn is_running(&self) -> bool;

    /// Sends one command and waits for the reply.
    fn send(&self, command: &Command) -> Result<Response, IpcError>;
}

impl<T: IpcClient> Daemon for T {
    fn is_running(&self) -> bool {
        self.is_daemon_running()
    }

    fn send(&self, command: &Command) -> Result<Response, IpcError> {
        IpcClient::send(self, command)
    }
}

/// The time source commands poll with.
pub trait Clock {
    /// The current time.
    fn now(&self) -> Instant;

    /// Waits for `duration`.
    fn sleep(&self, duration: Duration);
}

/// The real clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// Everything a command may use.
pub struct Context<'a> {
    /// The daemon of this instance.
    pub daemon: &'a dyn Daemon,
    /// Starting, checking and ending processes.
    pub processes: &'a dyn ProcessControl,
    /// Start at login.
    pub autostart: &'a dyn Autostart,
    /// Moving windows on the desktop.
    ///
    /// Positions and sizes are in the coordinates the daemon works in
    /// only when the command runs through `platform::run_on_desktop`,
    /// which makes the process DPI aware first. Without it, a window
    /// placed on a scaled monitor lands in the wrong place.
    pub windows: &'a dyn WindowControl,
    /// Where the daemon records its process id.
    pub pid_file: PidFile,
    /// The instance's config directory, for telling the user.
    pub config_dir: PathBuf,
    /// Time, for polling.
    pub clock: &'a dyn Clock,
    /// Looks up a newer release, returning its tag.
    pub check_update: &'a dyn Fn() -> Option<String>,
    /// Draws the mosaico logo.
    pub logo: &'a dyn Fn(&mut dyn Write),
    /// Standard output.
    pub out: &'a mut dyn Write,
    /// Standard error.
    pub err: &'a mut dyn Write,
}

/// Writes a line to a command's standard output. Output errors are
/// ignored, as `println!` would panic on them, which is worse.
macro_rules! say {
    ($cx:expr, $($arg:tt)*) => {{
        let _ = ::std::io::Write::write_fmt(
            &mut *$cx.out,
            format_args!("{}\n", format_args!($($arg)*)),
        );
    }};
}

/// Writes a line to a command's standard error.
macro_rules! complain {
    ($cx:expr, $($arg:tt)*) => {{
        let _ = ::std::io::Write::write_fmt(
            &mut *$cx.err,
            format_args!("{}\n", format_args!($($arg)*)),
        );
    }};
}

pub(crate) use {complain, say};

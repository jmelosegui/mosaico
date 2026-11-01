//! What a command needs from the outside world.
//!
//! Commands take a [`Context`] instead of reaching for the daemon, the
//! process table, the registry, the desktop, the network, the clock or
//! the terminal directly, and return an [`Exit`] instead of ending the
//! process. `main` builds the real context and turns the exit into a
//! status code; the unit tests build one from fakes.

#[path = "context_clock.rs"]
mod clock;

use std::io::Write;
use std::path::PathBuf;

use mosaico_core::WindowControl;
use mosaico_core::config::ConfigStore;
use mosaico_core::http::HttpClient;
use mosaico_core::ipc::{Command, IpcClient, IpcError, Response};
use mosaico_core::pid::PidFile;
use mosaico_core::system::{Autostart, FileProbe, ProcessControl};
use mosaico_core::tiler::{Desktop, Enumerator};
use mosaico_core::update::Installer;

pub use clock::{Clock, SystemClock};

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
    /// Positions and sizes are in the coordinates the daemon works in,
    /// since `main` sets every process up first (`platform::init()`,
    /// which makes it DPI aware). Without that, a window placed on a
    /// scaled monitor would land in the wrong place.
    pub windows: &'a dyn WindowControl,
    /// The windows and monitors on the desktop.
    ///
    /// Positions and sizes are in the coordinates the daemon works in,
    /// and window frames mean something, since `main` sets every process
    /// up first (`platform::init()`, which makes it DPI aware), `doctor`
    /// included.
    pub desktop: &'a dyn Enumerator,
    /// The desktop as the tiler reads it, which also says whether the
    /// system lets windows be animated. It is made the first time a
    /// command asks for it, as only `doctor` does: it costs more to make
    /// than the rest of the context.
    pub tiler_desktop: &'a dyn Fn() -> &'a dyn Desktop,
    /// Finds the mosaico executable this command runs from, which is what
    /// autostart registers and what `update` replaces.
    pub current_exe: &'a dyn Fn() -> std::io::Result<PathBuf>,
    /// Looks at program files without waiting on the network, for telling
    /// whether autostart starts this mosaico.
    pub files: &'a dyn FileProbe,
    /// Where the daemon records its process id.
    pub pid_file: PidFile,
    /// The instance's config files.
    pub config: ConfigStore,
    /// Time, for polling.
    pub clock: &'a dyn Clock,
    /// The web, for looking up and downloading releases.
    pub http: &'a dyn HttpClient,
    /// How this platform's release is packaged and installed.
    pub installer: &'a dyn Installer,
    /// Draws the mosaico logo over `version`, as in `0.12.1`.
    pub logo: &'a dyn Fn(&mut dyn Write, &str),
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

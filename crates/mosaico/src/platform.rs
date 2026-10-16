//! Picks the platform implementation of each core contract.
//!
//! Commands use the core traits through a [`Context`] built here, and the
//! daemon through a [`Startup`], so supporting another OS means adding a
//! branch to this file rather than touching every command.

use mosaico_core::Instance;
use mosaico_core::config::ConfigStore;
use mosaico_core::event::EventSource;
use mosaico_core::instance::InstanceError;
use mosaico_core::lifecycle::Startup;
use mosaico_core::pid::PidFile;
use mosaico_core::system::{Interrupts, ProcessSetup};
use mosaico_core::tiler::Enumerator;

use crate::commands::banner;
use crate::commands::context::{Context, Exit, SystemClock};

/// Finds the daemon instance of this process from its environment and
/// the user's folders. `main` calls it before running any command.
#[cfg(target_os = "windows")]
pub fn resolve_instance() -> Result<&'static Instance, InstanceError> {
    Instance::init(&mosaico_windows::user_dirs::WindowsUserDirs::current())
}

/// The daemon instance this CLI talks to.
///
/// `main` resolves the instance before running any command, so this only
/// fails if that was skipped.
pub fn instance() -> &'static Instance {
    Instance::current().expect("the instance is resolved at startup")
}

/// Sets this process up for the platform, so it sees windows and
/// monitors the way the daemon does. `main` calls it once, before any
/// command runs, the daemon included.
#[cfg(target_os = "windows")]
pub fn init() {
    mosaico_windows::setup::WindowsSetup.init();
}

/// Runs `command` with the real daemon, processes, registry, desktop,
/// network, clock and terminal.
#[cfg(target_os = "windows")]
pub fn run(command: impl FnOnce(&mut Context) -> Exit) -> Exit {
    let instance = instance();
    let daemon = mosaico_windows::ipc::PipeClient::new(&mosaico_windows::ipc::pipe_name(instance));
    let mut out = std::io::stdout();
    let mut err = std::io::stderr();
    let mut cx = Context {
        daemon: &daemon,
        processes: &mosaico_windows::process::WindowsProcesses,
        autostart: &mosaico_windows::autostart::RegistryAutostart,
        windows: &mosaico_windows::window_control::Win32WindowControl::default(),
        desktop: &mosaico_windows::enumerate::WindowsEnumerator,
        current_exe: &std::env::current_exe,
        files: &mosaico_windows::files::WindowsFiles,
        pid_file: PidFile::new(instance.pid_path()),
        config: ConfigStore::for_instance(instance),
        clock: &SystemClock,
        http: &mosaico_windows::http::WinHttpClient,
        installer: &mosaico_windows::install::RenameInstaller,
        logo: &banner::draw_logo,
        out: &mut out,
        err: &mut err,
    };
    command(&mut cx)
}

/// What the daemon of this instance starts and stops with: the real
/// single instance lock, installer and executable, and the platform's
/// daemon loop.
#[cfg(target_os = "windows")]
pub fn daemon_startup() -> Startup<'static> {
    Startup {
        instance: instance(),
        lock: &mosaico_windows::instance_guard::NamedMutex,
        installer: &mosaico_windows::install::RenameInstaller,
        exe: std::env::current_exe().ok(),
        pid: std::process::id(),
        serve: &mosaico_windows::daemon::serve,
    }
}

/// Where window events come from, for `debug events`.
#[cfg(target_os = "windows")]
pub fn event_source() -> impl EventSource {
    mosaico_windows::event_source::WindowsEventSource
}

/// How this process hears about Ctrl+C, for `debug events`.
#[cfg(target_os = "windows")]
pub fn interrupts() -> impl Interrupts {
    mosaico_windows::ctrl_c::ConsoleInterrupts
}

/// The windows on the desktop, for the titles `debug events` shows.
#[cfg(target_os = "windows")]
pub fn desktop() -> impl Enumerator {
    mosaico_windows::enumerate::WindowsEnumerator
}

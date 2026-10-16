//! Picks the platform implementation of each core contract.
//!
//! Commands use the core traits through a [`Context`] built here, so
//! supporting another OS means adding a branch to this file rather than
//! touching every command.

use mosaico_core::Instance;
use mosaico_core::pid::PidFile;

use crate::commands::banner;
use crate::commands::context::{Context, Exit, SystemClock};

/// The daemon instance this CLI talks to.
///
/// `main` checks the instance before running any command, so this only
/// fails if that check was skipped.
pub fn instance() -> &'static Instance {
    Instance::current().expect("the instance is validated at startup")
}

/// Runs `command` with the real daemon, processes, registry, desktop,
/// clock and terminal.
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
        windows: &mosaico_windows::window_control::Win32WindowControl,
        pid_file: PidFile::new(instance.pid_path()),
        config_dir: instance.config_dir().to_path_buf(),
        clock: &SystemClock,
        check_update: &mosaico_windows::version_check::check_for_update,
        logo: &banner::draw_logo,
        out: &mut out,
        err: &mut err,
    };
    command(&mut cx)
}

/// Runs `command` like [`run`], for commands that move windows, after
/// making this process see the desktop in the physical pixels the
/// daemon uses.
///
/// Without per-monitor DPI awareness Windows scales the coordinates of
/// this process on high-DPI monitors, so a window would not land where
/// the daemon would put it.
#[cfg(target_os = "windows")]
pub fn run_on_desktop(command: impl FnOnce(&mut Context) -> Exit) -> Exit {
    mosaico_windows::dpi::enable_dpi_awareness();
    run(command)
}

//! The real config, autostart entry and daemon of this instance.

use std::path::PathBuf;
use std::time::Duration;

use mosaico_core::config::ConfigStore;
use mosaico_core::ipc::{DaemonStatus, IpcClient};
use mosaico_core::system::{
    Autostart, AutostartStatus, InstanceLock, LaunchCommand, SingleWindow, Summoned, font_choices,
};
use mosaico_core::tiler::Enumerator;
use mosaico_core::{Command, Error, Instance};
use mosaico_settings::{DaemonState, Services};
use mosaico_windows::autostart::RegistryAutostart;
use mosaico_windows::enumerate::WindowsEnumerator;
use mosaico_windows::files::WindowsFiles;
use mosaico_windows::fonts::InstalledFonts;
use mosaico_windows::ipc::{PipeClient, pipe_name};
use mosaico_windows::single_window::NamedWindowLock;
use mosaico_windows::user_dirs::WindowsUserDirs;

/// How long the window waits for the daemon to answer.
const DAEMON_TIMEOUT: Duration = Duration::from_millis(300);

/// What the settings window's lock is called, beside the instance.
const APP: &str = "settings";

/// Takes the lock of this instance's settings window, with `summoned`
/// called when a later launch asks it to come forward. Fails with
/// [`Error::AlreadyRunning`] when another window holds it, which was
/// asked to come forward.
pub fn lock_window(summoned: Summoned) -> mosaico_core::Result<InstanceLock> {
    let instance = Instance::init(&WindowsUserDirs::current())
        .map_err(|e| Error::platform("find the instance", e))?;
    NamedWindowLock.acquire(instance, APP, summoned)
}

/// The settings of this process's instance. When the instance cannot be
/// told, the window shows the defaults and cannot save.
pub fn services() -> Services {
    let Ok(instance) = Instance::init(&WindowsUserDirs::current()) else {
        return Services {
            store: Err("The mosaico config folder could not be found.".into()),
            start_at_sign_in: false,
            sign_in: Box::new(|_| Err("The mosaico executable could not be found.".into())),
            log_file: String::new(),
            monitors: 1,
            fonts: Vec::new(),
            daemon: Box::new(|| DaemonState::Stopped),
        };
    };
    let autostart = AutostartStatus::read(&RegistryAutostart, &WindowsFiles, mosaico_exe);
    let client = PipeClient::new(&pipe_name(instance));
    Services {
        store: Ok(ConfigStore::for_instance(instance)),
        start_at_sign_in: autostart.is_ok_and(|s| s == AutostartStatus::Enabled),
        sign_in: Box::new(|on| set_sign_in(on).map_err(|e| e.to_string())),
        log_file: instance.log_path().display().to_string(),
        monitors: WindowsEnumerator.monitors().map_or(1, |m| m.len().max(1)),
        fonts: font_choices(&InstalledFonts, ""),
        daemon: Box::new(move || daemon_state(&client)),
    }
}

/// Registers mosaico to start at sign-in, or removes it, as `mosaico
/// autostart enable` and `disable` do.
fn set_sign_in(on: bool) -> mosaico_core::Result<()> {
    if on {
        RegistryAutostart.enable(&LaunchCommand::autostart(mosaico_exe()?))
    } else {
        RegistryAutostart.disable()
    }
}

fn daemon_state(client: &PipeClient) -> DaemonState {
    client
        .send_with_timeout(&Command::Status, DAEMON_TIMEOUT)
        .ok()
        .and_then(|response| response.data::<DaemonStatus>())
        .map_or(DaemonState::Stopped, |status| DaemonState::Running {
            version: status.version,
        })
}

/// The mosaico executable installed beside this one, which is what
/// starts at sign-in.
fn mosaico_exe() -> mosaico_core::Result<PathBuf> {
    let exe = std::env::current_exe().map_err(|e| Error::platform("find this executable", e))?;
    Ok(exe.with_file_name(format!("mosaico{}", std::env::consts::EXE_SUFFIX)))
}

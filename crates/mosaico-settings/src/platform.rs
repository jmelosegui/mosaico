//! The real config, autostart entry and daemon of this instance.

use std::path::PathBuf;
use std::time::Duration;

use mosaico_core::config::ConfigStore;
use mosaico_core::ipc::{DaemonStatus, IpcClient};
use mosaico_core::system::AutostartStatus;
use mosaico_core::{Command, Error, Instance};
use mosaico_settings::{DaemonState, General, Services};
use mosaico_windows::autostart::RegistryAutostart;
use mosaico_windows::files::WindowsFiles;
use mosaico_windows::ipc::{PipeClient, pipe_name};
use mosaico_windows::user_dirs::WindowsUserDirs;

/// How long the window waits for the daemon to answer.
const DAEMON_TIMEOUT: Duration = Duration::from_millis(300);

/// The settings of this process's instance, falling back to the defaults
/// when the instance cannot be told.
pub fn services() -> Services {
    let Ok(instance) = Instance::init(&WindowsUserDirs::current()) else {
        return Services {
            general: General::defaults(),
            log_file: String::new(),
            daemon: Box::new(|| DaemonState::Stopped),
        };
    };
    let config = ConfigStore::for_instance(instance).load_config();
    let autostart = AutostartStatus::read(&RegistryAutostart, &WindowsFiles, mosaico_exe);
    let client = PipeClient::new(&pipe_name(instance));
    Services {
        general: General::from_config(
            &config,
            autostart.is_ok_and(|s| s == AutostartStatus::Enabled),
        ),
        log_file: instance.log_path().display().to_string(),
        daemon: Box::new(move || daemon_state(&client)),
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

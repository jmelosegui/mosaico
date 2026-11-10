//! What the daemon does with what the user picks on the tray icon.

use mosaico_core::Action;
use mosaico_core::action::Origin;
use mosaico_core::bar::BarHost;
use mosaico_core::config::ConfigReload;
use mosaico_core::tray::{TrayCommand, TrayNotice};

use crate::tiling::TilingManager;

use super::daemon_inputs::Inputs;
use super::daemon_loop_commands::carry_out;
use super::daemon_loop_handlers::handle_reload;

/// Carries out `command`, and says whether the daemon should stop. What
/// could not be done is logged and shown beside the icon.
pub(super) fn handle(
    command: TrayCommand,
    manager: &mut TilingManager,
    bars: &mut BarHost,
    inputs: &mut Inputs,
    hotkeys_paused: &mut bool,
) -> bool {
    mosaico_core::log_info!("Tray: {command:?}");
    let done = match command {
        TrayCommand::OpenSettings => inputs.shortcuts().open_settings(),
        TrayCommand::OpenConfigFolder => inputs.shortcuts().open_config_folder(),
        TrayCommand::OpenLog => inputs.shortcuts().open_log(),
        TrayCommand::TogglePause => {
            // carry_out has logged a failure, and the icon shows the state.
            let _ = carry_out(
                Action::TogglePause,
                Origin::Focused,
                manager,
                bars,
                inputs,
                hotkeys_paused,
            );
            Ok(())
        }
        TrayCommand::ReloadConfig => {
            let reloads = inputs.reload_all();
            let all_loaded = !reloads
                .iter()
                .any(|reload| matches!(reload, ConfigReload::Invalid { .. }));
            for reload in reloads {
                handle_reload(reload, manager, bars, inputs);
            }
            if all_loaded {
                inputs.notify(&TrayNotice::reloaded());
            }
            Ok(())
        }
        TrayCommand::Quit => return true,
    };
    if let Err(notice) = done {
        mosaico_core::log_warn!("{}: {}", notice.title, notice.body);
        inputs.notify(&notice);
    }
    false
}

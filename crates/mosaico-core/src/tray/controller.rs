//! Keeping the icon in step with the config and the daemon.

use std::sync::mpsc::Sender;

use super::{TrayCommand, TrayNotice, TraySource, TrayState, TrayWatch};

/// The icon of a running daemon: shown while the config enables it, and
/// showing whether the hotkeys are paused.
///
/// A problem with the icon never stops the daemon: it is logged, and the
/// daemon goes on without it.
pub struct Tray {
    source: Box<dyn TraySource>,
    commands: Sender<TrayCommand>,
    state: TrayState,
    watch: Option<TrayWatch>,
}

impl Tray {
    /// The icon of a daemon of `version`, not shown yet. What the user
    /// picks on it goes to `commands`.
    pub fn new(
        source: impl TraySource + 'static,
        commands: Sender<TrayCommand>,
        version: &str,
    ) -> Self {
        Self {
            source: Box::new(source),
            commands,
            state: TrayState::new(version),
            watch: None,
        }
    }

    /// Whether the icon is on screen.
    pub fn is_shown(&self) -> bool {
        self.watch.is_some()
    }

    /// Shows the icon, or removes it. Showing it again shows the latest
    /// state.
    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            if let Some(watch) = self.watch.take() {
                watch.stop();
                crate::log_info!("Tray icon removed");
            }
            return;
        }
        if self.watch.is_some() {
            return;
        }
        match self.source.start(&self.state, self.commands.clone()) {
            Ok(watch) => self.watch = Some(watch),
            Err(e) => crate::log_warn!("Could not show the tray icon: {e}"),
        }
    }

    /// Shows whether the hotkeys are paused.
    pub fn set_paused(&mut self, paused: bool) {
        if self.state.paused == paused {
            return;
        }
        self.state.paused = paused;
        if let Some(watch) = &self.watch
            && let Err(e) = watch.update(&self.state)
        {
            crate::log_warn!("Could not update the tray icon: {e}");
        }
    }

    /// Shows `notice` beside the icon, when it is shown.
    pub fn notify(&self, notice: &TrayNotice) {
        if let Some(watch) = &self.watch
            && let Err(e) = watch.notify(notice)
        {
            crate::log_warn!("Could not show a tray notification: {e}");
        }
    }

    /// Removes the icon and waits until it is gone.
    pub fn stop(mut self) {
        self.set_enabled(false);
    }
}

#[cfg(test)]
#[path = "tests/controller_tests.rs"]
mod tests;

//! Where the icon comes from.

use std::sync::mpsc::Sender;

use crate::Result;

use super::{TrayCommand, TrayNotice, TrayState};

/// Shows the mosaico icon in the notification area and sends what the
/// user picks on it.
///
/// # Contract for platform implementations
///
/// - `start` shows the icon with the tooltip, the icon variant and the
///   menu of `state`, and returns once it is shown. It fails, and sends
///   nothing, when the icon cannot be shown.
/// - A click on the icon sends [`TrayCommand::DEFAULT`]. The menu, opened
///   the way the platform opens the menus of these icons, lists
///   [`TrayState::menu`] in order with the default entry marked, and
///   picking an entry sends its command. Commands are sent in the order
///   the user picked them.
/// - The icon has a variant for running and one for paused, which is
///   dimmed and badged so the two can be told apart at a glance.
/// - When the shell that hosts the icon restarts, the icon comes back
///   with the latest state, without being asked.
/// - [`TrayControl::update`] and [`TrayControl::notify`] return without
///   waiting, and the source applies them in the order it was asked to.
/// - Once the watch has stopped, the icon is gone, nothing more is sent
///   and the source has dropped `commands`. A receiver that went away is
///   not an error: commands are dropped until the watch stops. `start`
///   may be called again after a watch stopped.
pub trait TraySource {
    /// Shows the icon for `state`, starts sending what the user picks to
    /// `commands`, and returns once the icon is shown.
    fn start(&self, state: &TrayState, commands: Sender<TrayCommand>) -> Result<TrayWatch>;
}

/// How a platform updates and removes the icon it showed.
///
/// A [`TrayWatch`] holds it, so that the source is stopped exactly once,
/// by [`TrayWatch::stop`] or on drop.
pub trait TrayControl: Send {
    /// Shows `state`: the icon variant, the tooltip and the menu labels.
    ///
    /// # Errors
    ///
    /// When the request cannot reach the source.
    fn update(&self, state: &TrayState) -> Result<()>;

    /// Shows `notice` beside the icon. The platform may shorten it to
    /// what its notifications hold.
    ///
    /// # Errors
    ///
    /// When the request cannot reach the source.
    fn notify(&self, notice: &TrayNotice) -> Result<()>;

    /// Removes the icon. Must not return before the icon is gone and the
    /// source has stopped sending and dropped its sender.
    fn stop(self: Box<Self>);
}

/// A running [`TraySource`]. Dropping it removes the icon, as
/// [`stop`](Self::stop) does.
#[must_use = "dropping the watch removes the icon"]
pub struct TrayWatch {
    control: Option<Box<dyn TrayControl>>,
}

impl TrayWatch {
    /// Wraps how a platform controls the icon it showed.
    pub fn new(control: impl TrayControl + 'static) -> Self {
        Self {
            control: Some(Box::new(control)),
        }
    }

    /// Shows `state` on the icon.
    ///
    /// # Errors
    ///
    /// When the request cannot reach the source, which then shows what it
    /// showed before.
    pub fn update(&self, state: &TrayState) -> Result<()> {
        self.control
            .as_ref()
            .map_or(Ok(()), |control| control.update(state))
    }

    /// Shows `notice` beside the icon.
    ///
    /// # Errors
    ///
    /// When the request cannot reach the source.
    pub fn notify(&self, notice: &TrayNotice) -> Result<()> {
        self.control
            .as_ref()
            .map_or(Ok(()), |control| control.notify(notice))
    }

    /// Removes the icon and waits until it is gone.
    pub fn stop(self) {
        drop(self);
    }
}

impl Drop for TrayWatch {
    fn drop(&mut self) {
        if let Some(control) = self.control.take() {
            control.stop();
        }
    }
}

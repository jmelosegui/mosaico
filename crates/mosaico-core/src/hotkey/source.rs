//! Where the actions of the hotkeys come from.

use std::sync::mpsc::Sender;

use crate::config::Keybinding;
use crate::{Action, Result};

/// Registers the hotkeys of the keybindings and sends the action of each
/// press, the way [`EventSource`](crate::event::EventSource) sends window
/// events.
///
/// # Contract for platform implementations
///
/// - `start` registers the bindings through [`Hotkeys`](super::Hotkeys)
///   over the platform's [`HotkeyRegistrar`](super::HotkeyRegistrar), so
///   which bindings are left out and what a pause keeps are decided in
///   core, and returns once they are registered.
/// - A binding that cannot be registered is logged with the reason and
///   left out, and the others work. `start` fails, and sends nothing,
///   only when the platform cannot watch hotkeys at all.
/// - The action of every press that
///   [`Hotkeys::triggered`](super::Hotkeys::triggered) lets through is
///   sent to `actions`, in the order of the presses.
/// - [`HotkeyWatch::pause`] and [`HotkeyWatch::unpause`] return without
///   waiting. The source pauses and unpauses in the order it was asked
///   to, and logs the hotkeys that [`Hotkeys::pause`](super::Hotkeys::pause)
///   and [`Hotkeys::unpause`](super::Hotkeys::unpause) report.
/// - Once the watch has stopped, nothing more is sent, every hotkey has
///   been given back and the source has dropped `actions`, so a loop over
///   the receiver ends. A receiver that went away is not an error:
///   actions are dropped until the watch stops. `start` may be called
///   again after a watch stopped.
pub trait HotkeySource {
    /// Registers `bindings`, starts sending the actions of their presses
    /// to `actions`, and returns once they are registered.
    fn start(&self, bindings: Vec<Keybinding>, actions: Sender<Action>) -> Result<HotkeyWatch>;
}

/// How a platform pauses, unpauses and stops the hotkeys it started.
///
/// A [`HotkeyWatch`] holds it, so that the source is stopped exactly once,
/// by [`HotkeyWatch::stop`] or on drop.
pub trait HotkeyControl: Send {
    /// Asks the source to pause its hotkeys.
    ///
    /// # Errors
    ///
    /// When the request cannot reach the source.
    fn pause(&self) -> Result<()>;

    /// Asks the source to unpause its hotkeys.
    ///
    /// # Errors
    ///
    /// When the request cannot reach the source.
    fn unpause(&self) -> Result<()>;

    /// Stops the source. Must not return before the source has given back
    /// its hotkeys, stopped sending and dropped its sender.
    fn stop(self: Box<Self>);
}

/// A running [`HotkeySource`]. Dropping it stops the source, as
/// [`stop`](Self::stop) does.
#[must_use = "dropping the watch stops the hotkeys"]
pub struct HotkeyWatch {
    control: Option<Box<dyn HotkeyControl>>,
}

impl HotkeyWatch {
    /// Wraps how a platform controls the hotkeys it started.
    pub fn new(control: impl HotkeyControl + 'static) -> Self {
        Self {
            control: Some(Box::new(control)),
        }
    }

    /// Gives every hotkey back to the applications except the ones bound
    /// to `toggle-pause`, until [`unpause`](Self::unpause).
    ///
    /// # Errors
    ///
    /// When the request cannot reach the source, which then stays as it
    /// was.
    pub fn pause(&self) -> Result<()> {
        self.control
            .as_ref()
            .map_or(Ok(()), |control| control.pause())
    }

    /// Registers again what [`pause`](Self::pause) gave back.
    ///
    /// # Errors
    ///
    /// When the request cannot reach the source, which then stays as it
    /// was.
    pub fn unpause(&self) -> Result<()> {
        self.control
            .as_ref()
            .map_or(Ok(()), |control| control.unpause())
    }

    /// Stops the source and waits until it has stopped.
    pub fn stop(self) {
        drop(self);
    }
}

impl Drop for HotkeyWatch {
    fn drop(&mut self) {
        if let Some(control) = self.control.take() {
            control.stop();
        }
    }
}

#[cfg(test)]
#[path = "tests/source_tests.rs"]
mod tests;

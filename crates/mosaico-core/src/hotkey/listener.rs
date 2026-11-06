//! The hotkeys the daemon listens to, started over with other bindings
//! when they change.

use std::sync::mpsc::Sender;

use crate::Result;
use crate::config::{Keybinding, LaunchBinding};

use super::{HotkeySource, HotkeyWatch, Target};

/// Runs a [`HotkeySource`] for the daemon: starts it, pauses and unpauses
/// it, and starts it over with other bindings while the daemon runs.
///
/// A [`Target::Launch`] counts in the launch bindings it was last started
/// with. Starting over stops the running source first, so the new bindings
/// can take the keys of the old ones, and pauses the new source when the
/// hotkeys are paused. A press the old source sent just before it stopped
/// is read against the new launch bindings.
pub struct HotkeyListener<S: HotkeySource> {
    source: S,
    presses: Sender<Target>,
    watch: Option<HotkeyWatch>,
    launches: Vec<LaunchBinding>,
    paused: bool,
}

impl<S: HotkeySource> HotkeyListener<S> {
    /// Starts `source` with `bindings` and `launches`, sending what their
    /// presses ask for to `presses`.
    ///
    /// # Errors
    ///
    /// When the source cannot start, as [`HotkeySource::start`] says.
    pub fn start(
        source: S,
        bindings: Vec<Keybinding>,
        launches: Vec<LaunchBinding>,
        presses: Sender<Target>,
    ) -> Result<Self> {
        let watch = source.start(bindings, launches.clone(), presses.clone())?;
        Ok(Self {
            source,
            presses,
            watch: Some(watch),
            launches,
            paused: false,
        })
    }

    /// The launch binding a [`Target::Launch`] with `index` names.
    pub fn launch(&self, index: usize) -> Option<&LaunchBinding> {
        self.launches.get(index)
    }

    /// Pauses the hotkeys, as [`HotkeyWatch::pause`] does, until
    /// [`unpause`](Self::unpause), including across a
    /// [`restart`](Self::restart).
    ///
    /// # Errors
    ///
    /// When the request cannot reach the source.
    pub fn pause(&mut self) -> Result<()> {
        self.paused = true;
        self.watch.as_ref().map_or(Ok(()), HotkeyWatch::pause)
    }

    /// Registers again what [`pause`](Self::pause) gave back.
    ///
    /// # Errors
    ///
    /// When the request cannot reach the source.
    pub fn unpause(&mut self) -> Result<()> {
        self.paused = false;
        self.watch.as_ref().map_or(Ok(()), HotkeyWatch::unpause)
    }

    /// Stops the source and starts it again with `bindings` and
    /// `launches`, paused when the hotkeys are.
    ///
    /// # Errors
    ///
    /// When the source cannot start again, in which case no hotkey works
    /// until a later restart succeeds, or when the new source cannot be
    /// paused, in which case it runs unpaused.
    pub fn restart(
        &mut self,
        bindings: Vec<Keybinding>,
        launches: Vec<LaunchBinding>,
    ) -> Result<()> {
        if let Some(watch) = self.watch.take() {
            watch.stop();
        }
        self.launches = launches;
        let watch = self
            .source
            .start(bindings, self.launches.clone(), self.presses.clone())?;
        let watch = self.watch.insert(watch);
        if self.paused {
            watch.pause()?;
        }
        Ok(())
    }

    /// Stops the source and waits until it has stopped.
    pub fn stop(self) {
        drop(self);
    }
}

#[cfg(test)]
#[path = "tests/listener_tests.rs"]
mod tests;

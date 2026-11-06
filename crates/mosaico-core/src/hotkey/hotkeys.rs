//! Which hotkeys are registered when: at the start, while paused and at
//! the end.

use crate::config::{Keybinding, LaunchBinding};
use crate::{Action, Error, Result};

use super::{HotkeyId, HotkeyRegistrar, KeyCombo, Rejected, Target};

/// The hotkeys of the configured keybindings and launch bindings, and the
/// rule for pausing them.
///
/// Pausing gives every hotkey back to the applications except the ones
/// bound to `toggle-pause`, so the user can still unpause from the
/// keyboard, and unpausing registers again exactly what the pause gave
/// back. Dropping it gives back every hotkey still registered.
pub struct Hotkeys<R: HotkeyRegistrar> {
    registrar: R,
    hotkeys: Vec<Hotkey>,
    paused: bool,
}

/// A binding that was registered at the start.
struct Hotkey {
    id: HotkeyId,
    combo: KeyCombo,
    target: Target,
    /// What the binding does, for the reasons it is rejected.
    name: String,
    /// Whether the registrar holds it right now.
    registered: bool,
}

impl Hotkey {
    /// Whether it stays registered while paused, to unpause with.
    fn survives_pause(&self) -> bool {
        self.target == Target::Action(Action::TogglePause)
    }

    fn rejected(&self, error: Error) -> Rejected {
        Rejected::new(self.name.clone(), Some(self.combo), error)
    }
}

impl<R: HotkeyRegistrar> Hotkeys<R> {
    /// Registers the hotkeys of `bindings` and then of `launches` through
    /// `registrar`.
    ///
    /// A binding whose key name is unknown, or whose combination cannot
    /// be registered, is left out and returned with the reason. The other
    /// bindings work all the same, so of two bindings with the same
    /// combination, the first one wins.
    pub fn register(
        registrar: R,
        bindings: &[Keybinding],
        launches: &[LaunchBinding],
    ) -> (Self, Vec<Rejected>) {
        let mut hotkeys = Self {
            registrar,
            hotkeys: Vec::new(),
            paused: false,
        };
        let actions = bindings.iter().map(|binding| {
            let target = Target::Action(binding.action);
            (target, binding.action.to_string(), binding.combo())
        });
        let launches = launches
            .iter()
            .enumerate()
            .map(|(index, launch)| (Target::Launch(index), launch.name(), launch.combo()));
        let rejected = actions
            .chain(launches)
            .enumerate()
            .filter_map(|(position, (target, name, combo))| {
                hotkeys.add(position, target, name, combo).err()
            })
            .collect();
        (hotkeys, rejected)
    }

    /// Registers the binding at `position` among all the bindings.
    fn add(
        &mut self,
        position: usize,
        target: Target,
        name: String,
        combo: Result<KeyCombo>,
    ) -> std::result::Result<(), Rejected> {
        let combo = match combo {
            Ok(combo) => combo,
            Err(error) => return Err(Rejected::new(name, None, error)),
        };
        let id = match u16::try_from(position + 1) {
            Ok(raw) => HotkeyId::from_raw(raw),
            Err(_) => {
                let error = Error::config(None, "too many keybindings to register");
                return Err(Rejected::new(name, Some(combo), error));
            }
        };
        if let Err(error) = self.registrar.register(id, combo) {
            return Err(Rejected::new(name, Some(combo), error));
        }
        self.hotkeys.push(Hotkey {
            id,
            combo,
            target,
            name,
            registered: true,
        });
        Ok(())
    }

    /// What to do for a press of the hotkey registered with `id`.
    ///
    /// `None` when `id` is not one of these hotkeys and, while paused, for
    /// the hotkeys the pause gave back, even for a press that was already
    /// on its way when the pause came. A hotkey that is still registered
    /// while paused, such as `toggle-pause`, keeps running its action.
    pub fn triggered(&self, id: HotkeyId) -> Option<Target> {
        let hotkey = self.hotkeys.iter().find(|hotkey| hotkey.id == id)?;
        (!self.paused || hotkey.registered).then_some(hotkey.target)
    }

    /// Whether the hotkeys are paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Gives every hotkey back to the applications except the ones bound
    /// to `toggle-pause`.
    ///
    /// Does nothing when already paused. With no `toggle-pause` binding,
    /// nothing stays registered and only `mosaico unpause` can unpause.
    /// Returns the hotkeys that could not be given back. They stay
    /// registered, so their keystrokes still do not reach the
    /// applications, and they keep running their actions.
    pub fn pause(&mut self) -> Vec<Rejected> {
        if self.paused {
            return Vec::new();
        }
        self.paused = true;
        let mut rejected = Vec::new();
        for hotkey in &mut self.hotkeys {
            if !hotkey.registered || hotkey.survives_pause() {
                continue;
            }
            match self.registrar.unregister(hotkey.id) {
                Ok(()) => hotkey.registered = false,
                Err(error) => rejected.push(hotkey.rejected(error)),
            }
        }
        rejected
    }

    /// Registers again what [`pause`](Self::pause) gave back.
    ///
    /// Does nothing when not paused. Returns the hotkeys that could not be
    /// registered again, as when another program took the combination in
    /// the meantime; each is tried again at the next unpause.
    pub fn unpause(&mut self) -> Vec<Rejected> {
        if !self.paused {
            return Vec::new();
        }
        self.paused = false;
        let mut rejected = Vec::new();
        for hotkey in self.hotkeys.iter_mut().filter(|hotkey| !hotkey.registered) {
            match self.registrar.register(hotkey.id, hotkey.combo) {
                Ok(()) => hotkey.registered = true,
                Err(error) => rejected.push(hotkey.rejected(error)),
            }
        }
        rejected
    }
}

/// Gives back every hotkey still registered.
impl<R: HotkeyRegistrar> Drop for Hotkeys<R> {
    fn drop(&mut self) {
        for hotkey in self.hotkeys.iter().filter(|hotkey| hotkey.registered) {
            // Nothing is left to retry a failure with.
            let _ = self.registrar.unregister(hotkey.id);
        }
    }
}

#[cfg(test)]
#[path = "tests/hotkeys_tests.rs"]
mod tests;

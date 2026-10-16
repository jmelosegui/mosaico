//! A registrar that keeps its hotkeys in memory, for the hotkey tests.

use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

use crate::config::{Keybinding, Modifier};
use crate::hotkey::{HotkeyId, HotkeyRegistrar, KeyCombo, Modifiers, letter};
use crate::{Action, Direction, Error, Result};

/// What the fake system holds.
#[derive(Default)]
pub(super) struct Desktop {
    /// The combinations registered now, by id.
    registered: BTreeMap<HotkeyId, KeyCombo>,
    /// Every call made, oldest first.
    calls: Vec<Call>,
    /// Combinations that another program holds.
    pub(super) taken: HashSet<KeyCombo>,
    /// Combinations the system refuses to give back.
    pub(super) stuck: HashSet<KeyCombo>,
}

/// A call made to the registrar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Call {
    Register(HotkeyId),
    Unregister(HotkeyId),
}

/// The registrar. Clones share one [`Desktop`], so a test keeps a clone
/// to look at what the hotkeys under test did.
#[derive(Clone, Default)]
pub(super) struct FakeRegistrar(Rc<RefCell<Desktop>>);

impl FakeRegistrar {
    pub(super) fn desktop(&self) -> RefMut<'_, Desktop> {
        self.0.borrow_mut()
    }

    /// The combinations registered now, in the order of their ids.
    pub(super) fn held(&self) -> Vec<KeyCombo> {
        self.desktop().registered.values().copied().collect()
    }

    /// The id `combo` is registered with now.
    pub(super) fn id_of(&self, combo: KeyCombo) -> HotkeyId {
        let desktop = self.desktop();
        let found = desktop.registered.iter().find(|(_, held)| **held == combo);
        *found.expect("the combination is registered").0
    }

    /// The calls made since the last time this was asked.
    pub(super) fn take_calls(&self) -> Vec<Call> {
        std::mem::take(&mut self.desktop().calls)
    }
}

impl HotkeyRegistrar for FakeRegistrar {
    fn register(&mut self, id: HotkeyId, combo: KeyCombo) -> Result<()> {
        let mut desktop = self.desktop();
        desktop.calls.push(Call::Register(id));
        let held = desktop.registered.values().any(|held| *held == combo);
        if held || desktop.taken.contains(&combo) || desktop.registered.contains_key(&id) {
            return Err(Error::platform("register a hotkey", "it is taken"));
        }
        desktop.registered.insert(id, combo);
        Ok(())
    }

    fn unregister(&mut self, id: HotkeyId) -> Result<()> {
        let mut desktop = self.desktop();
        desktop.calls.push(Call::Unregister(id));
        let stuck = match desktop.registered.get(&id) {
            Some(combo) => desktop.stuck.contains(combo),
            None => {
                return Err(Error::platform(
                    "unregister a hotkey",
                    "it is not registered",
                ));
            }
        };
        if stuck {
            return Err(Error::platform("unregister a hotkey", "the system kept it"));
        }
        desktop.registered.remove(&id);
        Ok(())
    }
}

pub(super) fn bind(action: Action, key: &str, modifiers: &[Modifier]) -> Keybinding {
    Keybinding {
        action,
        key: key.into(),
        modifiers: modifiers.to_vec(),
    }
}

pub(super) fn alt_j() -> KeyCombo {
    KeyCombo {
        modifiers: Modifiers::ALT,
        key: letter('J'),
    }
}

pub(super) fn alt_k() -> KeyCombo {
    KeyCombo {
        modifiers: Modifiers::ALT,
        key: letter('K'),
    }
}

pub(super) fn alt_shift(upper_case: char) -> KeyCombo {
    KeyCombo {
        modifiers: Modifiers::ALT | Modifiers::SHIFT,
        key: letter(upper_case),
    }
}

/// Alt+J focuses down, Alt+K focuses up, Alt+Shift+R retiles and
/// Alt+Shift+P toggles the pause.
pub(super) fn bindings() -> Vec<Keybinding> {
    use Modifier::{Alt, Shift};

    vec![
        bind(Action::Focus(Direction::Down), "J", &[Alt]),
        bind(Action::Focus(Direction::Up), "K", &[Alt]),
        bind(Action::Retile, "R", &[Alt, Shift]),
        bind(Action::TogglePause, "P", &[Alt, Shift]),
    ]
}

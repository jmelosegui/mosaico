//! The contract a platform implements to claim hotkeys.

use crate::Result;

use super::KeyCombo;

/// Identifies one registered hotkey, so that a press can be traced back
/// to its binding.
///
/// [`Hotkeys`](super::Hotkeys) picks the ids and hands each to the
/// platform with its combination. A platform registers the hotkey under
/// [`raw`](Self::raw), and uses [`from_raw`](Self::from_raw) only to turn
/// the number a press was reported with back into the id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HotkeyId(u16);

impl HotkeyId {
    /// The id whose [`raw`](Self::raw) number is `raw`.
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// The number the hotkey is registered under, and its presses are
    /// reported with.
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// Claims key combinations for the whole desktop, so that pressing one
/// reaches mosaico whichever application has the focus.
///
/// [`Hotkeys`](super::Hotkeys) decides what is registered and when,
/// including what stays registered while paused. A registrar only
/// carries that out.
///
/// # Contract for platform implementations
///
/// - `register` claims `combo` for the whole desktop. Until it is
///   unregistered, every press of it is reported to
///   [`Hotkeys::triggered`](super::Hotkeys::triggered) with `id`, and the
///   application with the focus does not get the keystroke.
/// - Holding the keys down is one press, not a stream of repeats.
/// - `register` fails, and claims nothing, when another program or
///   another id holds the combination, or when the platform cannot watch
///   it: [`Error::Platform`](crate::Error::Platform) with what the system
///   reported, or [`Error::Unsupported`](crate::Error::Unsupported) for a
///   key the platform has no way to watch.
/// - `unregister` gives the combination back to the applications. The
///   same id and combination can be registered again afterwards.
/// - Ids are unique among the hotkeys registered at a time, and no
///   larger than the number of bindings, so a platform may register them
///   as its own ids.
/// - A registrar is only used on the thread that created it, and presses
///   must be reported on that thread too. It need not be `Send`, so it may
///   rely on state the system ties to that thread.
/// - A registrar may keep what it needs to give a registration back and
///   to report its presses, such as a native handle per id, but no pause
///   state and no actions: what is registered when is up to
///   [`Hotkeys`](super::Hotkeys).
pub trait HotkeyRegistrar {
    /// Claims `combo`, and reports its presses with `id`.
    fn register(&mut self, id: HotkeyId, combo: KeyCombo) -> Result<()>;

    /// Gives back the combination registered with `id`.
    fn unregister(&mut self, id: HotkeyId) -> Result<()>;
}

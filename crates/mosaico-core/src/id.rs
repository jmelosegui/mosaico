use std::fmt;

use serde::{Deserialize, Serialize};

/// Identifies a window for the length of a session.
///
/// Opaque to the core: ids are compared, ordered, hashed, printed and
/// stored, but the order means nothing beyond being stable. Each platform
/// builds an id from its own native handle and turns it back into one.
///
/// # Contract for platform implementations
///
/// - Two live windows never share an id.
/// - The id of a window does not change while the window exists.
/// - An id may be reused once its window is gone, so it must not be
///   kept across sessions.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WindowId(u64);

impl WindowId {
    /// Wraps a platform handle.
    ///
    /// Only platform implementations, and the fakes that stand in for
    /// them in tests, build ids from raw handles. Everywhere else an id
    /// comes from the platform and is compared, hashed, printed, or read
    /// back from its printed form or from where it was stored or sent,
    /// never made up. "No window" is `None`, not an id of `0`.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The platform handle this id was built from.
    ///
    /// For platform implementations only, to turn an id back into a
    /// native handle. Nothing else may read meaning into the number.
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Reads an id back the way [`Display`](fmt::Display) prints it,
    /// such as `0x4014E` (prefix and digits in either case), or written
    /// in decimal. Anything else, a sign or spaces included, is refused.
    ///
    /// This is for ids a user types, such as one copied from `debug
    /// list`. Besides the platform's [`WindowId::from_raw`] and reading
    /// an id back from where it was stored or sent, it is the only way to
    /// build an id. It only reads the text: whether a window has that id
    /// is for the platform to find out.
    pub fn parse(text: &str) -> Option<Self> {
        let (digits, radix) = match text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
            Some(hex) => (hex, 16),
            None => (text, 10),
        };
        // The integer parser takes a leading `+`, which `Display` never
        // prints.
        if digits.starts_with('+') {
            return None;
        }
        u64::from_str_radix(digits, radix).ok().map(Self)
    }
}

/// Printed in hexadecimal, the usual way window handles are shown.
impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

/// The printed id inside the type name, so a window reads the same in
/// logs, `debug` output and test failures.
impl fmt::Debug for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WindowId({self})")
    }
}

/// Identifies a connected monitor until the next display change.
///
/// Opaque to the core: ids are compared, ordered, hashed and printed, so
/// they can key a map, but the order means nothing. Platforms may hand
/// out new ids on every display change, even for monitors that stayed
/// connected, so use [`crate::MonitorKey`] to recognise a monitor across
/// changes.
///
/// # Contract for platform implementations
///
/// - Two connected monitors never share an id.
/// - The id of a monitor does not change between display changes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MonitorId(u64);

impl MonitorId {
    /// Wraps a platform handle.
    ///
    /// Only platform implementations, and the fakes that stand in for
    /// them in tests, build ids from raw handles. Everywhere else an id
    /// comes from the platform and is compared, hashed or printed, never
    /// made up.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The platform handle this id was built from.
    ///
    /// For platform implementations only, to turn an id back into a
    /// native handle. Nothing else may read meaning into the number.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Printed in hexadecimal, like window ids.
impl fmt::Display for MonitorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

/// The printed id inside the type name, as for [`WindowId`].
impl fmt::Debug for MonitorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MonitorId({self})")
    }
}

/// Shorthand for a window id in tests.
#[cfg(test)]
pub(crate) fn wid(raw: u64) -> WindowId {
    WindowId::from_raw(raw)
}

/// Shorthand for a list of window ids in tests.
#[cfg(test)]
pub(crate) fn wids(raw: &[u64]) -> Vec<WindowId> {
    raw.iter().copied().map(WindowId::from_raw).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_id_round_trips_and_prints_in_hex() {
        // Arrange
        let id = WindowId::from_raw(0x4014E);

        // Act / Assert
        assert_eq!(id.raw(), 0x4014E);
        assert_eq!(id.to_string(), "0x4014E");
    }

    #[test]
    fn window_id_parses_what_it_prints_and_decimal() {
        // Arrange
        let id = WindowId::from_raw(0x4014E);

        // Act / Assert
        assert_eq!(WindowId::parse(&id.to_string()), Some(id));
        assert_eq!(WindowId::parse("0X4014e"), Some(id));
        assert_eq!(WindowId::parse("262478"), Some(id));
    }

    #[test]
    fn window_id_refuses_what_it_never_prints() {
        // Arrange -- including the signs the integer parser would take.
        let texts = [
            "", "0x", "0x4014G", "-1", "+262478", "0x+4014E", "notepad", " 0x4014E",
        ];

        // Act / Assert
        for text in texts {
            assert_eq!(WindowId::parse(text), None, "{text:?}");
        }
    }

    #[test]
    fn monitor_id_round_trips_and_prints_in_hex() {
        // Arrange
        let id = MonitorId::from_raw(0x1830091);

        // Act / Assert
        assert_eq!(id.raw(), 0x1830091);
        assert_eq!(id.to_string(), "0x1830091");
    }

    #[test]
    fn debug_shows_the_same_number_as_display() {
        // Arrange
        let window = WindowId::from_raw(0x4014E);
        let monitor = MonitorId::from_raw(0x10001);

        // Act / Assert
        assert_eq!(format!("{window:?}"), "WindowId(0x4014E)");
        assert_eq!(format!("{monitor:?}"), "MonitorId(0x10001)");
        assert_eq!(format!("{:?}", Some(window)), "Some(WindowId(0x4014E))");
    }

    #[test]
    fn a_window_id_is_stored_as_a_plain_number() {
        // Arrange
        let id = WindowId::from_raw(0x4014E);

        // Act
        let json = serde_json::to_string(&id).unwrap();
        let back: WindowId = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(json, "262478");
        assert_eq!(back, id);
    }
}

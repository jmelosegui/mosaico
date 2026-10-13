use std::fmt;

/// Identifies a window for the length of a session.
///
/// Opaque to the core: only compared, hashed and printed. Each platform
/// builds it from its own native handle and turns it back into one.
///
/// # Contract for platform implementations
///
/// - Two live windows never share an id.
/// - The id of a window does not change while the window exists.
/// - An id may be reused once its window is gone, so it must not be
///   kept across sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(u64);

impl WindowId {
    /// Wraps a platform handle.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The platform handle this id was built from.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Printed in hexadecimal, the usual way window handles are shown.
impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

/// Identifies a connected monitor until the next display change.
///
/// Opaque to the core: only compared and printed. Platforms may hand out
/// new ids on every display change, even for monitors that stayed
/// connected, so use [`crate::MonitorKey`] to recognise a monitor across
/// changes.
///
/// # Contract for platform implementations
///
/// - Two connected monitors never share an id.
/// - The id of a monitor does not change between display changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MonitorId(u64);

impl MonitorId {
    /// Wraps a platform handle.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The platform handle this id was built from.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for MonitorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
    fn monitor_id_round_trips() {
        // Arrange
        let id = MonitorId::from_raw(25362577);

        // Act / Assert
        assert_eq!(id.raw(), 25362577);
        assert_eq!(id.to_string(), "25362577");
    }
}

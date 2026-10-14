//! What a platform sets up in a mosaico process before anything else.

/// Process-wide setup a platform needs before mosaico looks at windows
/// and monitors.
///
/// # Contract for platform implementations
///
/// - `init` is called once, first thing in every mosaico process, the
///   daemon and each CLI command alike, before any other call into the
///   platform.
/// - Afterwards the positions and sizes of windows and monitors that the
///   platform reports, and those it applies, are in the same units in
///   every mosaico process, whatever the display scaling. A position one
///   process reads, such as a window `debug list` shows, means the same
///   in the daemon that tiles it.
/// - It cannot fail. What the platform cannot set up, for example because
///   it was settled earlier for the process, is left as it is, and a
///   second call changes nothing.
/// - A platform with nothing to set up does nothing.
pub trait ProcessSetup {
    /// Sets this process up for the platform.
    fn init(&self);
}

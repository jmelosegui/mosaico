//! What the CLI asks of the OS besides talking to the daemon: starting
//! and stopping processes, and registering for start on login.

/// Starting and stopping daemon processes.
///
/// # Contract for platform implementations
///
/// - `launch_daemon` starts a daemon of the current instance in the
///   background and returns its process id without waiting for it. The
///   daemon must not be tied to the caller's console: closing the
///   terminal or pressing Ctrl+C there must not stop it, and the caller
///   must not wait on handles the daemon inherited. It inherits the
///   caller's environment, so it belongs to the same instance.
/// - `is_alive` answers for any process id, not only daemons; an id that
///   was never used, or whose process has exited, is not alive.
/// - `kill` ends the process without giving it a chance to clean up,
///   and returns whether it is gone.
pub trait ProcessControl {
    /// Starts a daemon in the background and returns its process id.
    fn launch_daemon(&self) -> Result<u32, String>;

    /// Whether a process with this id is running.
    fn is_alive(&self, pid: u32) -> bool;

    /// Ends a process forcefully. Returns whether it was ended.
    fn kill(&self, pid: u32) -> bool;
}

/// Starting mosaico when the user logs in.
///
/// # Contract for platform implementations
///
/// - Registration is per user and needs no elevation.
/// - What is registered runs `mosaico start` with the executable the
///   caller is running, so a moved or updated binary is picked up by
///   enabling again.
/// - `disable` succeeds when nothing was registered.
/// - A platform without a way to start at login returns an error from
///   `enable` explaining that, and `is_enabled` returns false.
pub trait Autostart {
    /// Whether mosaico is registered to start at login.
    fn is_enabled(&self) -> bool;

    /// Registers mosaico to start at login.
    fn enable(&self) -> Result<(), String>;

    /// Removes the registration.
    fn disable(&self) -> Result<(), String>;
}

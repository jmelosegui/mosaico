//! Keeping a second window of an app from opening.

use crate::{Instance, Result};

use super::InstanceLock;

/// What is called when a later launch of an app asks the running one to
/// come forward.
pub type Summoned = Box<dyn Fn() + Send + 'static>;

/// Lets one copy of an app of the mosaico family, such as the settings
/// window, run per instance, and lets a later launch bring that copy
/// forward instead of opening a second window.
///
/// # Contract for platform implementations
///
/// - `acquire` of `app` in `instance` takes the lock and returns it when
///   no process holds it. A lock is identified by the app and the
///   instance's name, so it never stands in the way of the daemon's
///   [`SingleInstance`](super::SingleInstance) lock, of another app, or
///   of the same app in another instance.
/// - While the lock is held, `acquire` of the same app and instance asks
///   the holder to come forward and fails at once with
///   [`Error::AlreadyRunning`](crate::Error::AlreadyRunning). The holder's
///   `summoned` runs, on a thread of the implementation's, once for every
///   such `acquire`, possibly after it returned. The failing process hands
///   the holder its right to bring a window to the front, where the
///   desktop only grants it to the app the user just started.
/// - The lock is released when the [`InstanceLock`] is dropped, which
///   waits until `summoned` will not run again, and when the process ends
///   in any other way, as the [`SingleInstance`](super::SingleInstance)
///   lock is.
/// - Any other failure is an [`Error::Platform`](crate::Error::Platform).
pub trait SingleWindow {
    /// Takes the lock of `app` in `instance`, calling `summoned` whenever
    /// a later launch asks this one to come forward.
    fn acquire(&self, instance: &Instance, app: &str, summoned: Summoned) -> Result<InstanceLock>;
}

//! Keeping a second daemon of an instance from starting.

use std::fmt;

use crate::{Instance, Result};

/// The lock that lets only one daemon of an instance run at a time.
///
/// # Contract for platform implementations
///
/// - While an [`InstanceLock`] of an instance is held, `acquire` of the
///   same instance fails at once rather than wait for the lock. It fails
///   with [`Error::AlreadyRunning`](crate::Error::AlreadyRunning) when
///   this process may open the lock, as it always may when it holds the
///   lock itself, and with [`Error::Platform`](crate::Error::Platform)
///   when it may not, as when another user's daemon, or one with more
///   rights than this process, holds it.
/// - A lock is identified by the instance's name alone, as its IPC
///   endpoint is ([`Instance::endpoint_name`]). Instances with different
///   names lock independently, and two with the same name share one lock
///   even when their directories differ, since they would share one
///   endpoint too. What identifies the lock stays the same from one
///   release to the next, so a daemon of an older release still keeps a
///   newer one from starting.
/// - The lock is released when the [`InstanceLock`] is dropped, and when
///   the process ends in any other way, such as a crash, being killed or
///   a panic that aborts, with none of its code running. A daemon that
///   died must never keep the next one from starting, so the mere
///   existence of a file, such as a PID file, does not qualify, while a
///   lock the OS releases along with the process that holds it does.
/// - Any other failure is an [`Error::Platform`](crate::Error::Platform).
pub trait SingleInstance {
    /// Takes the lock of `instance` for this process.
    fn acquire(&self, instance: &Instance) -> Result<InstanceLock>;
}

/// A held [`SingleInstance`] lock. Dropping it releases the lock.
#[must_use = "dropping the lock lets another daemon start"]
pub struct InstanceLock {
    release: Option<Box<dyn FnOnce()>>,
}

impl InstanceLock {
    /// Wraps how a platform releases its lock. `release` runs once, when
    /// the lock is dropped.
    pub fn new(release: impl FnOnce() + 'static) -> Self {
        Self {
            release: Some(Box::new(release)),
        }
    }
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            release();
        }
    }
}

impl fmt::Debug for InstanceLock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstanceLock").finish_non_exhaustive()
    }
}

#[cfg(test)]
#[path = "tests/single_instance_tests.rs"]
mod tests;

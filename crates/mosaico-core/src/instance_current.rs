//! The instance of this process, resolved once at startup.

use std::ffi::OsString;
use std::sync::OnceLock;

use super::{Instance, InstanceError};
use crate::user_dirs::UserDirs;

/// The instance of this process, once [`Instance::init`] has found it.
static CURRENT: Slot = Slot::new();

impl Instance {
    /// Resolves the instance of this process from its environment and
    /// the user's directories `dirs`, and makes it the one
    /// [`Instance::current`] returns.
    ///
    /// The CLI calls this at startup and refuses to run on an error, so
    /// a mistyped instance name can never fall back to the user's daemon.
    /// Once an instance is found, later calls return that one.
    pub fn init(dirs: &impl UserDirs) -> Result<&'static Instance, InstanceError> {
        CURRENT.init_from(|key| std::env::var_os(key), dirs)
    }

    /// The instance of this process, or `None` until [`Instance::init`]
    /// has found it.
    pub fn current() -> Option<&'static Instance> {
        CURRENT.get()
    }

    /// Fails with [`InstanceError::NotCurrent`] unless this is the
    /// instance of this process, the one [`Instance::current`] returns.
    ///
    /// For code that is given an instance but still finds some of its
    /// files through [`Instance::current`], such as the daemon.
    pub fn require_current(&self) -> Result<(), InstanceError> {
        CURRENT.require(self)
    }
}

/// Holds the instance of a process once it is resolved: [`CURRENT`] for
/// this process, or one of a test's own.
struct Slot(OnceLock<Instance>);

impl Slot {
    const fn new() -> Self {
        Self(OnceLock::new())
    }

    /// Resolves the instance from `var` and `dirs` into the slot, unless
    /// one is there already, and returns the one it holds. A failed
    /// resolution leaves the slot empty, so a later call can still
    /// succeed.
    fn init_from(
        &self,
        var: impl Fn(&str) -> Option<OsString>,
        dirs: &impl UserDirs,
    ) -> Result<&Instance, InstanceError> {
        if let Some(instance) = self.0.get() {
            return Ok(instance);
        }
        let instance = Instance::from_env(var, dirs)?;
        Ok(self.0.get_or_init(|| instance))
    }

    /// The instance the slot holds, if any.
    fn get(&self) -> Option<&Instance> {
        self.0.get()
    }

    /// Fails unless the slot holds `instance`.
    fn require(&self, instance: &Instance) -> Result<(), InstanceError> {
        if self.get() == Some(instance) {
            Ok(())
        } else {
            Err(InstanceError::NotCurrent)
        }
    }
}

#[cfg(test)]
#[path = "instance_current_tests.rs"]
mod tests;

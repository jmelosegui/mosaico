//! Single-instance daemon guard using a named kernel mutex.
//!
//! The first daemon creates the mutex; any subsequent daemon sees
//! `ERROR_ALREADY_EXISTS` and exits cleanly.

use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::HSTRING;

use mosaico_core::{Instance, WindowResult};

/// Namespace-qualified mutex name visible across all sessions.
const MUTEX_PREFIX: &str = "Global\\MosaicoDaemon";

/// The mutex that marks an instance's daemon as running:
/// `Global\MosaicoDaemon` for the default instance, with `-<name>`
/// appended for a named one.
pub fn mutex_name(instance: &Instance) -> String {
    match instance.name() {
        Some(name) => format!("{MUTEX_PREFIX}-{name}"),
        None => MUTEX_PREFIX.to_string(),
    }
}

/// RAII guard that holds a named kernel mutex for the process lifetime.
///
/// When dropped (on clean exit or panic unwind), the mutex is released
/// so the next daemon can start.
pub struct InstanceGuard {
    handle: HANDLE,
}

impl InstanceGuard {
    /// Attempts to acquire the single-instance mutex of `instance`.
    ///
    /// Returns `Ok(guard)` if no daemon of this instance runs yet, or an
    /// error if one already holds the mutex. Daemons of other instances
    /// do not get in the way.
    pub fn acquire(instance: &Instance) -> WindowResult<Self> {
        Self::acquire_named(&mutex_name(instance))
    }

    /// Acquires a mutex with the given name.
    ///
    /// This is the implementation behind [`acquire`]; it accepts an
    /// arbitrary name so unit tests can use isolated mutex names.
    fn acquire_named(name: &str) -> WindowResult<Self> {
        let name = HSTRING::from(name);

        // SAFETY: CreateMutexW creates or opens a named mutex.
        // `None` for security attributes, `false` for initial owner
        // since we only need it as a sentinel, not for synchronization.
        let handle = unsafe { CreateMutexW(None, false, &name) }?;

        // CreateMutexW succeeds even when the mutex already exists,
        // but sets the last error to ERROR_ALREADY_EXISTS.
        // SAFETY: GetLastError returns the error code set by the
        // immediately preceding Win32 call.
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            // SAFETY: We own this handle and must close it before
            // returning.
            unsafe {
                let _ = CloseHandle(handle);
            }
            return Err("Mosaico daemon is already running.".into());
        }

        Ok(Self { handle })
    }
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        // SAFETY: CloseHandle releases the mutex. This guard owns
        // the handle exclusively.
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_acquire_is_rejected() {
        // Arrange
        let _first = InstanceGuard::acquire_named("Local\\MosaicoTest_SecondAcquire")
            .expect("first acquire should succeed");

        // Act
        let second = InstanceGuard::acquire_named("Local\\MosaicoTest_SecondAcquire");

        // Assert
        assert!(second.is_err());
    }

    #[test]
    fn the_default_instance_keeps_the_usual_mutex() {
        // Arrange
        let default = Instance::new(None, "cfg", "data").unwrap();
        let named = Instance::new(Some("ci"), "cfg", "data").unwrap();

        // Act / Assert
        assert_eq!(mutex_name(&default), "Global\\MosaicoDaemon");
        assert_eq!(mutex_name(&named), "Global\\MosaicoDaemon-ci");
    }

    #[test]
    fn acquire_succeeds_after_release() {
        // Arrange
        let first = InstanceGuard::acquire_named("Local\\MosaicoTest_AfterRelease")
            .expect("first acquire should succeed");

        // Act
        drop(first);
        let second = InstanceGuard::acquire_named("Local\\MosaicoTest_AfterRelease");

        // Assert
        assert!(second.is_ok());
    }
}

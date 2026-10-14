//! The single instance lock, as a named kernel mutex.
//!
//! The first daemon of an instance creates the mutex; any later daemon of
//! the same instance sees `ERROR_ALREADY_EXISTS` and is turned away.

use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::HSTRING;

use mosaico_core::system::{InstanceLock, SingleInstance};
use mosaico_core::{Error, Instance, Result};

use crate::error::OrPlatform;

/// Namespace-qualified mutex name visible across all sessions.
const MUTEX_PREFIX: &str = "Global\\MosaicoDaemon";

/// [`SingleInstance`] on Windows: a named kernel mutex per instance.
///
/// The mutex is named after the instance's name only, and is in the
/// `Global\` namespace, so a daemon of an instance keeps a second one out
/// from any session on the machine. Windows closes every handle of a
/// process that ends, however it ends, and that is what releases the lock
/// of a daemon that crashed or was killed.
///
/// When the mutex exists but the security on it does not let this process
/// open it, as when another user's daemon or an elevated one holds it,
/// `CreateMutexW` fails with access denied. That is reported as the
/// [`Error::Platform`] it is, not as [`Error::AlreadyRunning`].
pub struct NamedMutex;

impl SingleInstance for NamedMutex {
    fn acquire(&self, instance: &Instance) -> Result<InstanceLock> {
        let guard = InstanceGuard::acquire_named(&mutex_name(instance))?;
        Ok(InstanceLock::new(move || drop(guard)))
    }
}

/// The mutex that marks an instance's daemon as running:
/// `Global\MosaicoDaemon` for the default instance, with `-<name>`
/// appended for a named one.
fn mutex_name(instance: &Instance) -> String {
    match instance.name() {
        Some(name) => format!("{MUTEX_PREFIX}-{name}"),
        None => MUTEX_PREFIX.to_string(),
    }
}

/// RAII guard that holds a named kernel mutex until it is dropped.
///
/// When dropped (on clean exit or panic unwind), the mutex is released
/// so the next daemon can start.
struct InstanceGuard {
    handle: HANDLE,
}

impl InstanceGuard {
    /// Creates the mutex `name`, or fails with [`Error::AlreadyRunning`]
    /// when it exists already.
    ///
    /// It takes any name so unit tests can use isolated mutex names.
    fn acquire_named(name: &str) -> Result<Self> {
        let name = HSTRING::from(name);

        // SAFETY: CreateMutexW creates or opens a named mutex.
        // `None` for security attributes, `false` for initial owner
        // since we only need it as a sentinel, not for synchronization.
        let handle = unsafe { CreateMutexW(None, false, &name) }
            .or_platform("create the single instance lock")?;

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
            return Err(Error::AlreadyRunning);
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
#[path = "instance_guard_tests.rs"]
mod tests;

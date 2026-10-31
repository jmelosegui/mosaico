//! The lock that keeps a second window of an app from opening, and the
//! event a later launch sets to bring the first one forward.

use std::sync::Arc;
use std::thread;

use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{CreateEventW, INFINITE, SetEvent, WaitForMultipleObjects};
use windows::Win32::UI::WindowsAndMessaging::{ASFW_ANY, AllowSetForegroundWindow};
use windows::core::{HSTRING, PCWSTR};

use mosaico_core::system::{InstanceLock, SingleWindow, Summoned};
use mosaico_core::{Error, Instance, Result};

use crate::error::OrPlatform;
use crate::instance_guard::InstanceGuard;

/// [`SingleWindow`] on Windows: a named mutex, and a named auto-reset
/// event beside it.
///
/// Both are in the session's `Local\` namespace, since a window shows on
/// the desktop of one session. A thread of the holder waits on the event
/// and runs `summoned` each time it is set. A later launch first lets any
/// process take the foreground (`AllowSetForegroundWindow`), which
/// Windows grants it as the process the user just started, and then sets
/// the event.
pub struct NamedWindowLock;

impl SingleWindow for NamedWindowLock {
    fn acquire(&self, instance: &Instance, app: &str, summoned: Summoned) -> Result<InstanceLock> {
        let name = lock_name(instance, app);
        // Both sides create the event, so a summons is never lost to the
        // holder not having created it yet.
        let summon = Event::named(&format!("{name}-summon"))?;
        let guard = match InstanceGuard::acquire_named(&name) {
            Ok(guard) => guard,
            Err(Error::AlreadyRunning) => {
                // SAFETY: AllowSetForegroundWindow only changes which
                // process may take the foreground next. It fails when
                // this process may not hand that right on, which leaves
                // the holder to come forward as far as Windows lets it.
                let _ = unsafe { AllowSetForegroundWindow(ASFW_ANY) };
                summon.set()?;
                return Err(Error::AlreadyRunning);
            }
            Err(e) => return Err(e),
        };
        let stop = Arc::new(Event::stop()?);
        let waiter = {
            let stop = stop.clone();
            thread::Builder::new()
                .name(format!("{app} summons"))
                .spawn(move || wait_for_summons(&summon, &stop, &summoned))
                .or_platform("start the summons thread")?
        };
        Ok(InstanceLock::new(move || {
            let _ = stop.set();
            let _ = waiter.join();
            drop(guard);
        }))
    }
}

/// `Local\Mosaico-<app>` for the default instance, with `-<name>`
/// appended for a named one.
fn lock_name(instance: &Instance, app: &str) -> String {
    match instance.name() {
        Some(name) => format!("Local\\Mosaico-{app}-{name}"),
        None => format!("Local\\Mosaico-{app}"),
    }
}

/// Runs `summoned` each time `summon` is set, until `stop` is.
fn wait_for_summons(summon: &Event, stop: &Event, summoned: &Summoned) {
    let handles = [summon.0, stop.0];
    // SAFETY: both handles stay open while this runs, since the caller
    // holds both events. Waiting for either of them changes nothing but
    // the auto-reset summon event.
    while unsafe { WaitForMultipleObjects(&handles, false, INFINITE) } == WAIT_OBJECT_0 {
        summoned();
    }
}

/// An event handle, closed on drop.
struct Event(HANDLE);

// SAFETY: an event handle may be waited on, set and closed from any
// thread, and nothing else is in the struct.
unsafe impl Send for Event {}
// SAFETY: as above; setting and waiting on an event are thread safe.
unsafe impl Sync for Event {}

impl Event {
    /// Creates or opens the auto-reset event `name`, which starts unset.
    fn named(name: &str) -> Result<Self> {
        // SAFETY: CreateEventW with default security and a valid wide
        // name returns a handle this struct then owns.
        unsafe { CreateEventW(None, false, false, &HSTRING::from(name)) }
            .map(Self)
            .or_platform("create the summon event")
    }

    /// A new unnamed event that stays set once set.
    fn stop() -> Result<Self> {
        // SAFETY: as in `named`, without a name.
        unsafe { CreateEventW(None, true, false, PCWSTR::null()) }
            .map(Self)
            .or_platform("create the stop event")
    }

    fn set(&self) -> Result<()> {
        // SAFETY: the handle is open while self lives.
        unsafe { SetEvent(self.0) }.or_platform("set an event")
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        // SAFETY: the struct owns the handle, which nothing uses after.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
#[path = "single_window_tests.rs"]
mod tests;

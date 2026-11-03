//! A single-threaded COM apartment for the calls that need one.

use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};

/// COM, entered on this thread for as long as it lives.
pub(crate) struct Apartment {
    /// Whether this entered COM, and so must leave it. It did not when
    /// the thread is already in another kind of apartment, and COM works
    /// there as it is.
    entered: bool,
}

impl Apartment {
    pub(crate) fn enter() -> Self {
        // SAFETY: CoInitializeEx may be called on any thread. A thread
        // already in COM gets S_FALSE, or RPC_E_CHANGED_MODE for another
        // kind of apartment.
        let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        Self {
            entered: result.is_ok(),
        }
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        if self.entered {
            // SAFETY: Paired with the CoInitializeEx in `enter` that
            // succeeded, on the same thread.
            unsafe { CoUninitialize() };
        }
    }
}

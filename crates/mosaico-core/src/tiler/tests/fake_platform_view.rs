//! What a test sees of a [`FakePlatform`]: its windows, and the changes
//! it was asked to make.

use crate::id::wid;

use super::super::FakeWindow;
use super::{Call, FakePlatform};

impl FakePlatform {
    /// A window as it is now.
    pub(crate) fn window(&self, id: u64) -> FakeWindow {
        self.shared.desktop.borrow().windows[&wid(id)].clone()
    }

    /// Closes a window, which leaves the desktop.
    pub(crate) fn close_window(&self, id: u64) {
        self.shared.desktop.borrow_mut().windows.remove(&wid(id));
    }

    /// Every change asked for so far.
    pub(crate) fn calls(&self) -> Vec<Call> {
        self.shared.calls.borrow().clone()
    }

    /// Every change asked for since the last call, forgetting them.
    pub(crate) fn take_calls(&self) -> Vec<Call> {
        std::mem::take(&mut *self.shared.calls.borrow_mut())
    }
}

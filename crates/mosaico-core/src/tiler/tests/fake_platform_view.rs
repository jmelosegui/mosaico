//! What a test sees of a [`FakePlatform`]: its windows, and the changes
//! it was asked to make.

use crate::WindowId;
use crate::id::wid;

use super::super::FakeWindow;
use super::{Call, FakePlatform};

impl FakePlatform {
    /// A window as it is now.
    pub(crate) fn window(&self, id: u64) -> FakeWindow {
        self.shared.desktop.borrow().windows[&wid(id)].clone()
    }

    /// Changes a window behind the manager's back, as the user or the
    /// application would.
    pub(crate) fn edit(&self, id: u64, change: impl FnOnce(&mut FakeWindow)) {
        let mut desktop = self.shared.desktop.borrow_mut();
        change(desktop.windows.get_mut(&wid(id)).expect("window exists"));
    }

    /// Puts the pointer at `at`, as the user moving it does.
    pub(crate) fn point_at(&self, at: (i32, i32)) {
        self.shared.desktop.borrow_mut().cursor = Some(at);
    }

    /// Closes a window, which leaves the desktop.
    pub(crate) fn close_window(&self, id: u64) {
        self.shared.desktop.borrow_mut().windows.remove(&wid(id));
    }

    /// The window the platform last gave focus to.
    pub(crate) fn focused(&self) -> Option<WindowId> {
        self.shared.desktop.borrow().focused
    }

    /// The windows with a border overlay, and whether each is showing.
    pub(crate) fn borders(&self) -> Vec<(u64, bool)> {
        let mut borders: Vec<(u64, bool)> = self
            .shared
            .desktop
            .borrow()
            .borders
            .iter()
            .map(|(w, &showing)| (w.raw(), showing))
            .collect();
        borders.sort_unstable();
        borders
    }

    /// The windows whose border overlay is showing.
    pub(crate) fn showing_borders(&self) -> Vec<u64> {
        self.borders()
            .into_iter()
            .filter(|&(_, showing)| showing)
            .map(|(w, _)| w)
            .collect()
    }

    /// Every change asked for so far.
    pub(crate) fn calls(&self) -> Vec<Call> {
        self.shared.calls.borrow().clone()
    }

    /// Every change asked for since the last call, forgetting them.
    pub(crate) fn take_calls(&self) -> Vec<Call> {
        std::mem::take(&mut *self.shared.calls.borrow_mut())
    }

    /// The windows whose border was drawn from scratch since the last
    /// call, in order, forgetting them. A border drawn again the same
    /// way, or only moved, keeps what was drawn.
    pub(crate) fn take_renders(&self) -> Vec<u64> {
        let renders = std::mem::take(&mut *self.shared.renders.borrow_mut());
        renders.into_iter().map(|w| w.raw()).collect()
    }
}

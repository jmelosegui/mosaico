//! A platform for the tiling manager, on the fake desktop.
//!
//! [`FakePlatform`] answers the manager's questions from a
//! [`FakeDesktop`] and makes the changes it asks for on it, the way a
//! platform changes real windows: placing a window moves it, hiding it
//! cloaks, hides or minimizes it, and a border shows up as an overlay,
//! drawn from scratch only when it would look different. It records
//! every change it is asked to make. Clones share the
//! desktop, so a test keeps one to look at the desktop, or to start
//! another manager on it the way the next daemon would.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::config::HidingBehaviour;
use crate::id::wid;
use crate::{Error, HidingSupport, Rect, Result, WindowId};

use super::super::{Display, Effect, LastDrawn};
use super::{FakeDesktop, FakeWindow, displays};

pub(crate) use calls::Call;

#[path = "fake_platform_calls.rs"]
mod calls;
#[path = "fake_platform_control.rs"]
mod control;
#[path = "fake_platform_desktop.rs"]
mod desktop;
#[path = "fake_platform_frames.rs"]
mod frames;
#[path = "fake_platform_view.rs"]
mod view;

/// What every clone of a [`FakePlatform`] shares.
struct Shared {
    desktop: RefCell<FakeDesktop>,
    displays: RefCell<Vec<Display>>,
    now: Cell<Instant>,
    calls: RefCell<Vec<Call>>,
    hiding: Cell<HidingSupport>,
    /// Whether listing the monitors fails.
    monitors_fail: Cell<bool>,
    /// Whether a move that does not wait lands at once.
    pre_places_land: Cell<bool>,
    /// How each border was last drawn.
    drawn: RefCell<HashMap<WindowId, LastDrawn>>,
    /// The windows whose border was drawn from scratch, in order.
    renders: RefCell<Vec<WindowId>>,
}

/// A platform on a fake desktop that records what it is asked to do.
#[derive(Clone)]
pub(crate) struct FakePlatform {
    shared: Rc<Shared>,
}

impl FakePlatform {
    /// `monitors` monitors side by side, with the given windows, each
    /// `(id, monitor)`, open on them. Every way of hiding is available.
    pub(crate) fn new(monitors: usize, windows: &[(u64, usize)]) -> Self {
        let mut desktop = FakeDesktop::default();
        for &(id, monitor) in windows {
            desktop.windows.insert(wid(id), FakeWindow::app(monitor));
            desktop.raise(wid(id));
        }
        Self {
            shared: Rc::new(Shared {
                desktop: RefCell::new(desktop),
                displays: RefCell::new(displays(monitors)),
                now: Cell::new(Instant::now()),
                calls: RefCell::new(Vec::new()),
                hiding: Cell::new(HidingSupport::default()),
                monitors_fail: Cell::new(false),
                pre_places_land: Cell::new(false),
                drawn: RefCell::new(HashMap::new()),
                renders: RefCell::new(Vec::new()),
            }),
        }
    }

    /// Takes away a way of hiding windows.
    pub(crate) fn lack(&self, missing: HidingBehaviour) {
        let had = self.shared.hiding.get();
        let hiding = HidingSupport::new(|how| {
            if how == missing {
                None
            } else {
                had.events(how)
            }
        });
        self.shared.hiding.set(hiding);
    }

    /// Makes listing the monitors fail.
    pub(crate) fn fail_monitors(&self) {
        self.shared.monitors_fail.set(true);
    }

    /// Makes a move that does not wait land at once, as it does when the
    /// application handles its messages straight away.
    pub(crate) fn land_pre_placements(&self) {
        self.shared.pre_places_land.set(true);
    }

    /// Moves the work area of monitor `index`.
    pub(crate) fn set_work_area(&self, index: usize, area: Rect) {
        self.shared.displays.borrow_mut()[index].work_area = area;
    }

    /// Lets time pass.
    pub(crate) fn advance(&self, by: Duration) {
        self.shared.now.set(self.shared.now.get() + by);
    }

    fn record(&self, call: Call) {
        self.shared.calls.borrow_mut().push(call);
    }

    /// Records `call` and changes the desktop the way `effect` says.
    fn change(&self, call: Call, effect: Effect) -> Result<()> {
        self.record(call);
        let now = self.shared.now.get();
        self.shared.desktop.borrow_mut().apply(&effect, now);
        Ok(())
    }

    /// Fails the way a platform does for a way of hiding it lacks.
    fn can_hide(&self, how: HidingBehaviour) -> Result<()> {
        match self.shared.hiding.get().events(how) {
            Some(_) => Ok(()),
            None => Err(Error::Unsupported("hiding windows that way")),
        }
    }
}

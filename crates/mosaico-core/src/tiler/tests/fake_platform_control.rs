//! What a [`FakePlatform`] does when the manager asks it to change a
//! window or move the pointer, and what it hands the manager. Borders and
//! corners are in `fake_platform_decorations.rs`.

use std::time::Instant;

use crate::config::HidingBehaviour;
use crate::{Decorations, HidingEvents, Rect, Result, WindowControl, WindowId};

use super::super::super::{Desktop, Effect, Enumerator, Platform, Timing};
use super::{Call, FakePlatform};

impl WindowControl for FakePlatform {
    fn place(&self, window: WindowId, rect: Rect) -> Result<()> {
        let effect = Effect::snap(window, rect, Timing::Now);
        self.change(Call::Place(window, rect), effect)
    }

    fn pre_place(&self, window: WindowId, rect: Rect) -> Result<()> {
        self.record(Call::PrePlace(window, rect));
        if self.shared.pre_places_land.get()
            && let Some(w) = self.shared.desktop.borrow_mut().windows.get_mut(&window)
        {
            w.rect = rect;
        }
        Ok(())
    }

    fn repaint(&self, window: WindowId) -> Result<()> {
        self.record(Call::Repaint(window));
        Ok(())
    }

    fn focus(&self, window: WindowId) -> Result<()> {
        let move_cursor = false;
        self.change(
            Call::Focus(window),
            Effect::Focus {
                window,
                move_cursor,
            },
        )
    }

    fn put_behind(&self, window: WindowId, other: WindowId) -> Result<()> {
        let effect = Effect::PutBehind { window, other };
        self.change(Call::PutBehind(window, other), effect)
    }

    fn close(&self, window: WindowId) -> Result<()> {
        self.record(Call::Close(window));
        Ok(())
    }

    fn minimize(&self, window: WindowId) -> Result<()> {
        self.change(Call::Minimize(window), Effect::Minimize { window })
    }

    fn hide(&self, window: WindowId, how: HidingBehaviour) -> Result<()> {
        self.can_hide(how)?;
        self.change(Call::Hide(window, how), Effect::Hide { window, how })
    }

    fn show(&self, window: WindowId, how: HidingBehaviour) -> Result<()> {
        self.can_hide(how)?;
        self.change(Call::Show(window, how), Effect::Show { window, how })
    }

    fn hiding_events(&self, how: HidingBehaviour) -> Option<HidingEvents> {
        self.shared.hiding.get().events(how)
    }

    fn reveal(&self, window: WindowId) -> Result<()> {
        self.change(Call::Reveal(window), Effect::Reveal { window })
    }

    fn move_cursor(&self, x: i32, y: i32) -> Result<()> {
        self.record(Call::MoveCursor(x, y));
        self.shared.desktop.borrow_mut().cursor = Some((x, y));
        Ok(())
    }
}

impl Platform for FakePlatform {
    fn desktop(&self) -> &dyn Desktop {
        self
    }

    fn control(&self) -> &dyn WindowControl {
        self
    }

    fn decorations(&mut self) -> &mut dyn Decorations {
        self
    }

    fn enumerator(&self) -> &dyn Enumerator {
        self
    }

    fn now(&self) -> Instant {
        self.shared.now.get()
    }
}

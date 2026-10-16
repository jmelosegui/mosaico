//! The desktop of the fake [`World`].
//!
//! Commands only place windows, so placements are recorded and can be
//! made to fail. Everything else succeeds and changes nothing.

use mosaico_core::config::HidingBehaviour;
use mosaico_core::{Error, Rect, Result, WindowControl, WindowId};

use super::World;

impl WindowControl for World {
    fn place(&self, window: WindowId, rect: Rect) -> Result<()> {
        if let Some(error) = self.place_error.borrow().clone() {
            return Err(Error::platform("move the window", error));
        }
        self.placed.borrow_mut().push((window, rect));
        Ok(())
    }

    fn pre_place(&self, _: WindowId, _: Rect) -> Result<()> {
        Ok(())
    }

    fn repaint(&self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn focus(&self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn close(&self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn minimize(&self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn hide(&self, _: WindowId, _: HidingBehaviour) -> Result<()> {
        Ok(())
    }

    fn show(&self, _: WindowId, _: HidingBehaviour) -> Result<()> {
        Ok(())
    }

    fn reveal(&self, _: WindowId) -> Result<()> {
        Ok(())
    }

    fn move_cursor(&self, _: i32, _: i32) -> Result<()> {
        Ok(())
    }
}

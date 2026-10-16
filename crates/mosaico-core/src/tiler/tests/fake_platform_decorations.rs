//! What a [`FakePlatform`] does when the manager asks it to draw, move,
//! hide or drop a border, or to change the corners of a window.

use crate::config::CornerStyle;
use crate::{Decorations, Rect, Result, WindowId};

use super::super::super::{BorderStyle, Effect};
use super::{Call, FakePlatform};

impl Decorations for FakePlatform {
    /// Draws the border from scratch only when it would look different
    /// from how it was last drawn.
    fn show_border(&mut self, window: WindowId, style: &BorderStyle) -> Result<()> {
        let mut drawn = self.shared.drawn.borrow_mut();
        let last = drawn.entry(window).or_default();
        if last.needs_render(style) {
            last.rendered(style, true);
            self.shared.renders.borrow_mut().push(window);
        }
        drop(drawn);
        let call = Call::ShowBorder(window, style.clone());
        let style = Some(style.clone());
        self.change(call, Effect::Border { window, style })
    }

    fn hide_border(&mut self, window: WindowId) -> Result<()> {
        let style = None;
        self.change(Call::HideBorder(window), Effect::Border { window, style })
    }

    /// Only records the move: the overlay keeps what it shows, and where
    /// it is plays no part in the fake.
    fn move_border(&mut self, window: WindowId, rect: Rect) -> Result<()> {
        self.record(Call::MoveBorder(window, rect));
        Ok(())
    }

    fn drop_border(&mut self, window: WindowId) -> Result<()> {
        self.shared.drawn.borrow_mut().remove(&window);
        self.change(Call::DropBorder(window), Effect::DropBorder { window })
    }

    fn forget_drawn(&mut self) {
        self.shared.drawn.borrow_mut().clear();
    }

    fn set_corners(&mut self, window: WindowId, corners: CornerStyle) -> Result<()> {
        self.record(Call::SetCorners(window, corners));
        Ok(())
    }

    fn reset_corners(&mut self, window: WindowId) -> Result<()> {
        self.record(Call::ResetCorners(window));
        Ok(())
    }
}

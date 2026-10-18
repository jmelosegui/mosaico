//! A bar as the bar manager uses it: drawn and shown, hidden, and
//! switched between taking the pointer and letting it through.

use mosaico_core::bar::{BarContent, BarSurface, ItemRegion};
use mosaico_core::config::bar::BarConfig;
use mosaico_core::{Rect, Result};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_TOP, SW_HIDE, SWP_NOACTIVATE, SWP_SHOWWINDOW, SetWindowPos, ShowWindow,
};

use super::input_os::let_through;
use super::render::render_bar;
use super::window::Bar;
use crate::error::OrPlatform;

impl BarSurface for Bar {
    fn draw(&mut self, config: &BarConfig, content: &BarContent) -> Result<Vec<ItemRegion>> {
        let monitor = self.monitor_rect;
        let area = Rect::new(monitor.x, monitor.y, monitor.width, config.height);
        let takes_input = !self.input.click_through();

        let rendered = render_bar(self.hwnd, area, config, content, takes_input);

        // SAFETY: SetWindowPos positions and shows the bar overlay.
        // The HWND is owned by this struct.
        let shown = unsafe {
            SetWindowPos(
                self.hwnd,
                Some(HWND_TOP),
                area.x,
                area.y,
                area.width,
                area.height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )
        }
        .or_platform("show the bar");

        let drawn = rendered.and_then(|regions| shown.map(|()| regions));
        let regions = drawn.as_ref().map_or_else(|_| Vec::new(), Clone::clone);
        self.input.set_regions(regions);
        drawn
    }

    fn hide(&mut self) -> Result<()> {
        self.input.set_regions(Vec::new());
        // SAFETY: ShowWindow hides the bar overlay owned by this struct.
        // What it returns is whether the bar was visible before, not
        // whether hiding worked, so there is nothing to report.
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
        Ok(())
    }

    /// Adds or removes `WS_EX_TRANSPARENT`. The next draw paints the
    /// clickable items for the new setting.
    fn set_click_through(&mut self, click_through: bool) -> Result<()> {
        let_through(self.hwnd, click_through)?;
        self.input.set_click_through(click_through);
        Ok(())
    }
}

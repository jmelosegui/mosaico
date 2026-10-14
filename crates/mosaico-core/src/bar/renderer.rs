//! The contracts a platform implements to put the bars on screen.

use crate::config::bar::BarConfig;
use crate::{Rect, Result};

use super::BarContent;

/// Makes the status bars, one per monitor.
///
/// [`BarManager`](super::BarManager) decides which monitors get a bar and
/// what each one shows. The renderer only puts them on screen.
///
/// # Contract for platform implementations
///
/// - [`create`](Self::create) makes a bar for the monitor whose work area
///   is `area` and returns it hidden. When the bar cannot be made it
///   returns why, and that monitor goes without one.
/// - A bar is a strip as tall as the config's `height` across the top of
///   its area, in front of application windows. It never takes focus or
///   pointer input, does not appear in the taskbar or the task switcher,
///   and is never reported to the tiler as a window to manage.
/// - Every bar is dropped before the renderer that made it, so a
///   renderer may release what its bars share, such as a loaded font or
///   a connection to the display, when it is dropped.
pub trait BarRenderer {
    /// Makes a hidden bar at the top of `area`.
    fn create(&mut self, area: Rect) -> Result<Box<dyn BarSurface>>;
}

/// One bar on screen.
///
/// # Contract for platform implementations
///
/// - [`draw`](Self::draw) paints `content` and shows the bar. Left items
///   run from the left edge to the right and right items from the right
///   edge to the left, both `padding` in from the edge, in the fonts,
///   sizes, gaps and colors of `config`. The widget lists are already
///   reflected in `content`, so the renderer does not read them.
/// - [`hide`](Self::hide) takes the bar off screen. The next `draw` shows
///   it again.
/// - A failed `draw` or `hide` is reported, and the next call tries
///   again.
/// - Dropping the bar removes it from the screen. Nothing else may: a
///   request from outside to close it, such as ending the task, must
///   leave it in place.
pub trait BarSurface {
    /// Paints `content` in the style of `config`, then shows the bar.
    fn draw(&mut self, config: &BarConfig, content: &BarContent) -> Result<()>;

    /// Takes the bar off screen until the next `draw`.
    fn hide(&mut self) -> Result<()>;
}

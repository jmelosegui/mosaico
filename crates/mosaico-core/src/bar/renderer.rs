//! The contracts a platform implements to put the bars on screen.

use std::rc::Rc;

use crate::config::bar::BarConfig;
use crate::{Rect, Result};

use super::{BarContent, BarId, ItemRegion, PointerSink};

/// Makes the status bars, one per monitor.
///
/// [`BarManager`](super::BarManager) decides which monitors get a bar and
/// what each one shows. The renderer only puts them on screen.
///
/// # Contract for platform implementations
///
/// - [`create`](Self::create) makes a bar for the monitor whose work area
///   is `area` and returns it hidden and letting the pointer through.
///   When the bar cannot be made it returns why, and that monitor goes
///   without one.
/// - A bar is a strip as tall as the config's `height` across the top of
///   its area, in front of application windows. It never takes focus,
///   whatever the pointer does on it, does not appear in the taskbar or
///   the task switcher, and is never reported to the tiler as a window to
///   manage.
/// - While a bar takes the pointer (see
///   [`BarSurface::set_click_through`]), it takes it over every item a
///   click does something on and wherever it draws something, and lets it
///   through to the windows under it where it draws nothing. It reports to
///   `sink`, under `id`, each press and release of a button, each turn of
///   the wheel over it and the pointer leaving it. A button pressed over
///   the bar keeps the pointer's input on the bar until it is released, so
///   the release is reported wherever it happens. Moves are not reported.
/// - Over an item that [`hit`](super::hit) finds clickable in the regions
///   the bar's last draw returned, the pointer shows a hand, and the usual
///   arrow elsewhere on the bar.
/// - A platform that cannot take pointer input leaves every bar letting
///   the pointer through, and reports nothing.
/// - Every bar is dropped before the renderer that made it, so a
///   renderer may release what its bars share, such as a loaded font or
///   a connection to the display, when it is dropped.
pub trait BarRenderer {
    /// Makes a hidden bar at the top of `area`, which reports what the
    /// pointer does on it to `sink` as the bar `id`.
    fn create(
        &mut self,
        id: BarId,
        area: Rect,
        sink: Rc<dyn PointerSink>,
    ) -> Result<Box<dyn BarSurface>>;
}

/// One bar on screen.
///
/// # Contract for platform implementations
///
/// - [`draw`](Self::draw) paints `content` and shows the bar. Left items
///   run from the left edge to the right and right items from the right
///   edge to the left, both `padding` in from the edge, in the fonts,
///   sizes, gaps and colors of `config`. The widget lists are already
///   reflected in `content`, so the renderer does not read them. The
///   right side only holds pills and separators.
/// - `draw` returns where it drew each item, in the order it drew them,
///   the left side first: for each item the region
///   [`BarContent::region`] gives at the rect it took, and after the
///   workspaces item, for each of its labels, the region
///   [`BarContent::label_region`] gives. Rects are in bar coordinates and
///   span the bar's height, so the pointer finds an item at the very top
///   of the screen.
/// - [`hide`](Self::hide) takes the bar off screen. The next `draw` shows
///   it again.
/// - While the bar takes the pointer, `draw` leaves no part of an item
///   that [`hit`](super::hit) finds clickable fully transparent: where the
///   background shows through, it paints the least opacity there is, which
///   no one can see, so that the whole item takes the pointer. Every other
///   part keeps the opacity it is drawn with, so the pointer goes through
///   wherever the bar is fully transparent there.
/// - A failed `draw` or `hide` is reported, and the next call tries
///   again.
/// - Dropping the bar removes it from the screen. Nothing else may: a
///   request from outside to close it, such as ending the task, must
///   leave it in place.
pub trait BarSurface {
    /// Paints `content` in the style of `config`, shows the bar, and
    /// returns where it drew each item.
    fn draw(&mut self, config: &BarConfig, content: &BarContent) -> Result<Vec<ItemRegion>>;

    /// Takes the bar off screen until the next `draw`.
    fn hide(&mut self) -> Result<()>;

    /// Lets the pointer through to the windows under the bar, or has the
    /// bar take it. The next `draw` paints the bar for the new setting.
    ///
    /// A platform that cannot take pointer input keeps this default, which
    /// changes nothing: its bars always let the pointer through.
    fn set_click_through(&mut self, click_through: bool) -> Result<()> {
        let _ = click_through;
        Ok(())
    }
}

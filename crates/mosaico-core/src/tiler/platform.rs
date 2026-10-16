//! What the tiling manager needs from the platform.

use std::time::Instant;

use crate::{Decorations, WindowControl};

use super::{Desktop, Enumerator};

/// Everything the platform does for the tiling manager: one
/// implementation of each contract the tiler is driven through, and the
/// time.
///
/// [`TilingManager`](super::TilingManager) runs a [`Tiler`](super::Tiler)
/// on the desktop. It reads windows and the pointer through
/// [`desktop`](Self::desktop) while the tiler decides, carries out the
/// effects the tiler returns through [`control`](Self::control) and
/// [`decorations`](Self::decorations), and lists windows and monitors
/// through [`enumerator`](Self::enumerator). A platform implements each
/// of those contracts and hands them over here.
///
/// # Contract for platform implementations
///
/// - Every accessor returns the same object each time it is called, for
///   as long as the manager runs. The manager asks
///   [`WindowControl::hiding_events`] once, when it starts, and the
///   decorations keep the borders they drew.
/// - All four describe the same desktop: the windows and monitors
///   [`enumerator`](Self::enumerator) lists are the ones the others take,
///   and rects mean the same to each of them.
/// - [`now`](Self::now) never goes backwards.
pub trait Platform {
    /// Answers the tiler's questions about windows, monitors and the
    /// pointer while it decides.
    fn desktop(&self) -> &dyn Desktop;

    /// Makes the changes to windows and the pointer the tiler asks for.
    fn control(&self) -> &dyn WindowControl;

    /// Draws the focus borders and the corners of windows.
    fn decorations(&mut self) -> &mut dyn Decorations;

    /// Lists windows and monitors: when the manager starts, and when it
    /// reads the work areas again.
    fn enumerator(&self) -> &dyn Enumerator;

    /// The time now. Every step is taken at the time this returns, so
    /// the time between two steps is the platform's to say.
    fn now(&self) -> Instant;
}

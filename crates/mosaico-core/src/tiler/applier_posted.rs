//! What may still be on its way to a window. While
//! [`posts_behind`](crate::frames::FrameControl::posts_behind) holds for
//! it, the platform posts a change behind what was posted to the window
//! before, and the desktop shows the change only once the window gets to
//! it. So the applier does not read such a change back as done:
//!
//! - A placement is not skipped because the desktop has the window in
//!   place already, unless the window was last sent there too, by a
//!   placement or a frame: what is still on its way would take it
//!   elsewhere.
//! - The pointer goes to the center of where the window was last sent,
//!   and a new motion of the window starts from there.
//! - A window a posted placement takes somewhere is repainted again once
//!   it lands there, and the border its motion kept, or the tiler decided
//!   meanwhile, is drawn there then (`applier_landing.rs`).
//! - A window a posted hide, minimize or show is to change has the show
//!   state it leaves the window in, in the tiler's view of the desktop,
//!   until the change lands (`applier_view.rs`).

use std::collections::{HashMap, HashSet};

use crate::frames::{Frame, FrameOutcome};
use crate::{Rect, WindowControl, WindowId};

use super::super::Platform;
use super::super::window_effects::place_and_repaint;
use super::Applier;
use super::landing::{LANDING_WAIT, Landing, is_there};
use super::shows::ShowState;

/// What the applier keeps of the changes that may be on their way.
#[derive(Debug, Default)]
pub(super) struct Posted {
    /// Where each window was last sent, by a placement or a frame.
    pub(super) sent: HashMap<WindowId, Rect>,
    /// The windows a posted placement is taking somewhere, until they land.
    pub(super) placed: HashMap<WindowId, Landing>,
    /// How the change of show state posted to each window leaves it, until
    /// it lands.
    pub(super) states: HashMap<WindowId, ShowState>,
    /// The windows given frames, until the platform no longer posts to
    /// them once transitions are off (`applier_stop.rs`).
    pub(super) framed: HashSet<WindowId>,
}

impl Posted {
    /// Forgets the change of show state posted to `window` that leaves it
    /// `as_reported`, which the window reports it got to. A report of
    /// another, such as a hide reported once a show was posted after it,
    /// leaves the later change on its way.
    pub(super) fn reported(&mut self, window: WindowId, as_reported: ShowState) {
        if self.states.get(&window) == Some(&as_reported) {
            self.states.remove(&window);
        }
    }

    /// Notes the windows given `frames`, and where the frames the platform
    /// handled sent them.
    pub(super) fn frames_sent(&mut self, frames: &[Frame], outcomes: &[FrameOutcome]) {
        for (frame, outcome) in frames.iter().zip(outcomes) {
            self.framed.insert(frame.window);
            if let FrameOutcome::Handled { .. } = outcome {
                self.sent.insert(frame.window, frame.rect);
            }
        }
    }
}

impl Applier {
    /// Places `window` at `rect` in full, unless the desktop has it there,
    /// not maximized, and nothing on its way to it may take it elsewhere:
    /// nothing may be, or the window was last sent there too.
    pub(super) fn place_unless_there(
        &mut self,
        window: WindowId,
        rect: Rect,
        platform: &dyn Platform,
    ) {
        let desktop = platform.desktop();
        let there = !desktop.is_maximized(window) && desktop.rect(window) == Some(rect);
        let sent_there = self.posted.sent.get(&window) == Some(&rect);
        if there && (sent_there || !platform.control().posts_behind(window)) {
            return;
        }
        self.send_to(window, rect, platform);
    }

    /// Places `window` at `rect` in full, wherever it is, and notes where
    /// it was sent. Says whether the window is still to land there: the
    /// placement was posted, as the platform says right after it, and the
    /// desktop does not have the window there yet, within 4 pixels, or the
    /// last thing sent to it took it elsewhere, by as little as a pixel, so
    /// this one moves it. Only one that is there, and was sent there last,
    /// is not moved, so it reports nothing, and needs no second repaint.
    /// What was kept for an earlier placement still on its way is dropped:
    /// the step that places the window decides its border again.
    pub(super) fn send_to(
        &mut self,
        window: WindowId,
        rect: Rect,
        platform: &dyn Platform,
    ) -> bool {
        place_and_repaint(window, rect, platform.control());
        let before = self.posted.sent.insert(window, rect);
        let posted = platform.control().posts_behind(window);
        let desktop = platform.desktop();
        let elsewhere = before.is_some_and(|sent| sent != rect);
        let landing = posted && (elsewhere || !is_there(desktop, window, rect));
        self.posted.placed.remove(&window);
        if landing {
            let from = desktop.rect(window);
            let until = Some(platform.now() + LANDING_WAIT);
            let placed = Landing {
                rect,
                from,
                border: None,
                until,
            };
            self.posted.placed.insert(window, placed);
        }
        landing
    }

    /// Where `window` was last sent, while what was sent to it may still
    /// be on its way, so that the desktop may not show it there yet.
    pub(super) fn last_sent(&self, window: WindowId, control: &dyn WindowControl) -> Option<Rect> {
        let sent = self.posted.sent.get(&window).copied()?;
        control.posts_behind(window).then_some(sent)
    }
}

//! What the mouse does in the tiler.

use crate::{Rect, WindowId};

use super::Drag;
use super::live::Live;

/// What the mouse does: whether the pointer follows focus, whether a
/// window dropped on a tile moves there, whether dragging the edge between
/// two tiles resizes them, the drag the tiler follows, if any, and the
/// tiles a drag laid out live.
#[derive(Debug, Default)]
pub(in crate::tiler) struct Mouse {
    /// Whether focus the tiler moves takes the pointer along.
    pub(in crate::tiler) follows_focus: bool,
    /// Whether a tiled window dropped on another tile takes its slot, or
    /// goes before it on another monitor (`[mouse] drag_to_swap`).
    pub(in crate::tiler) drag_to_swap: bool,
    /// Whether the edges of a tiled window the user resizes move the splits
    /// they border (`[mouse] resize_splits`).
    pub(in crate::tiler) resize_splits: bool,
    /// The drag the tiler follows.
    pub(super) drag: Option<Drag>,
    /// The tiles a drag laid out live, until they are back where their
    /// workspace has them (`drag_live.rs`).
    pub(super) live: Option<Live>,
    /// Where the drag the tiler follows last sent each window it laid out
    /// live, put back, or saw a layout pass send, which the desktop may not
    /// have it at yet.
    sent: Vec<(WindowId, Rect)>,
}

impl Mouse {
    /// The mouse with the pointer following focus when `follows_focus`,
    /// windows dropped on tiles moving there when `drag_to_swap`, and edges
    /// dragged moving splits when `resize_splits`.
    pub(in crate::tiler) fn new(
        follows_focus: bool,
        drag_to_swap: bool,
        resize_splits: bool,
    ) -> Self {
        Self {
            follows_focus,
            drag_to_swap,
            resize_splits,
            drag: None,
            live: None,
            sent: Vec::new(),
        }
    }

    /// Where the drag the tiler follows last sent `window`, if it did.
    pub(super) fn sent_to(&self, window: WindowId) -> Option<Rect> {
        (self.sent.iter())
            .find(|&&(w, _)| w == window)
            .map(|&(_, r)| r)
    }

    /// Notes that `window` was sent to `rect` while the tiler follows a
    /// drag, as a new record unless `known_only`, which only updates one.
    pub(super) fn note_sent(&mut self, window: WindowId, rect: Rect, known_only: bool) {
        match self.sent.iter_mut().find(|(w, _)| *w == window) {
            Some(at) => at.1 = rect,
            None if !known_only => self.sent.push((window, rect)),
            None => {}
        }
    }

    /// Forgets where windows were sent, as the drag is no longer followed.
    pub(super) fn forget_sent(&mut self) {
        self.sent.clear();
    }
}

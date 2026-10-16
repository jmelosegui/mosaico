//! What can be queued for a window of the fake desktop, and what it does
//! to the window when its application gets to it.

use crate::{Rect, WindowId};

use super::super::FakeWindow;

/// Something queued for a window, which its application gets to late.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Queued {
    /// A frame, to these bounds.
    Frame(Rect),
    /// A placement, to these bounds.
    Place(Rect),
    /// A placement, to these bounds, that leaves the window maximized:
    /// posted while it was maximized and to be out of view, and its
    /// application was not responding or it was still in view.
    PlaceMaximized(Rect),
    /// A minimize, or a hide by minimizing.
    Minimize,
    /// A hide that is not a minimize nor cloaking.
    Hide,
    /// A show of a window hidden by hiding or minimizing it.
    Show,
    /// A reveal.
    Reveal,
    /// A move right behind this window in the stacking order.
    PutBehind(WindowId),
}

impl Queued {
    /// Whether this change of the window's show state leaves it in view,
    /// or `None` for a change of something else.
    pub(super) fn in_view(self) -> Option<bool> {
        match self {
            Self::Minimize | Self::Hide => Some(false),
            Self::Show | Self::Reveal => Some(true),
            Self::Frame(_) | Self::Place(_) | Self::PlaceMaximized(_) | Self::PutBehind(_) => None,
        }
    }
}

/// What `queued` does to the window when its application gets to it.
pub(super) fn land(w: &mut FakeWindow, queued: Queued) {
    match queued {
        Queued::Frame(_) if w.absorbs_frames => {}
        Queued::Frame(rect) => {
            w.rect = rect;
            if !w.maximized {
                w.normal = rect;
            }
        }
        // As `WindowControl::place` promises, posted or not: no longer
        // maximized, but a minimized window stays minimized, and comes
        // back where it was before, as the contract allows.
        Queued::Place(rect) => {
            if !w.minimized {
                (w.rect, w.normal) = (rect, rect);
            }
            w.maximized = false;
        }
        Queued::PlaceMaximized(rect) => {
            if !w.minimized {
                w.rect = rect;
            }
        }
        Queued::Minimize => w.minimized = true,
        Queued::Hide => w.visible = false,
        // A minimized window comes back as it was, maximized or not, and a
        // maximized one goes back to its normal bounds.
        Queued::Show if !w.minimized && w.maximized => {
            (w.visible, w.maximized, w.rect) = (true, false, w.normal);
        }
        Queued::Show | Queued::Reveal => {
            w.visible = true;
            w.minimized = false;
        }
        // The stacking order is the desktop's, which lands it.
        Queued::PutBehind(_) => {}
    }
}

//! What happens to windows and displays, as the tiler hears of it.
//!
//! An [`EventSource`](crate::event::EventSource) reports what the
//! platform observes, and
//! [`FocusFollowsMouse`](crate::event::FocusFollowsMouse) reports where
//! the pointer comes to rest.

use crate::WindowId;

mod hover;
mod naming;
mod rest;
mod source;

pub use hover::{FocusFollowsMouse, MousePointer, POLL_INTERVAL, REST_POLLS};
pub use source::{EventSource, EventWatch};

/// Something that happened to a window or to the displays.
///
/// An event is a reason to look at the window again, not a statement of
/// its new state: the same change may be reported more than once, and
/// some reports name windows that are not worth managing. [`EventSource`]
/// lists what platforms promise about every event, and each variant says
/// when it is reported.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// A window was created or shown.
    ///
    /// Reported when a window is created, before it is first shown where
    /// the platform can tell, so that it can be put in place before its
    /// first frame, and every time it is shown. Some applications report
    /// it only for an element inside their window, under the window's id.
    /// A platform that only learns of windows once they are shown reports
    /// it then.
    Created {
        /// The window.
        window: WindowId,
    },

    /// A window no longer exists.
    ///
    /// Its id may be given to a new window afterwards. Also reported for
    /// windows that were never reported created.
    Destroyed {
        /// The window.
        window: WindowId,
    },

    /// A window stopped being shown without being minimized, as when an
    /// application closes to the tray.
    ///
    /// Also reported for a window that closes while it is shown, just
    /// before it is reported [`Destroyed`](Self::Destroyed), on platforms
    /// that hide a window as they destroy it. Not proof that it is gone
    /// either: some applications report a hide while they replace the
    /// surface they draw on, and stay on screen. Check that the window is
    /// no longer visible before taking it out of the layout. Minimizing
    /// reports [`Minimized`](Self::Minimized) instead.
    Hidden {
        /// The window.
        window: WindowId,
    },

    /// A window became the active one, which gets keyboard input.
    ///
    /// Reported for every window that becomes active, managed or not,
    /// including when the tiler focused it. Reports can arrive hundreds
    /// of milliseconds after the change, but in the order the changes
    /// happened, whichever windows they name: the last one reported is
    /// the window that has focus.
    Focused {
        /// The window.
        window: WindowId,
    },

    /// The pointer came to rest on a window.
    ///
    /// Sent by [`FocusFollowsMouse`], never by an [`EventSource`].
    MouseHover {
        /// The window.
        window: WindowId,
    },

    /// The user started moving or resizing a window by dragging it.
    ///
    /// Reported once, when the drag starts, and followed by
    /// [`Moved`](Self::Moved) when it ends, however it ends. Never
    /// reported for moves that programs make, the tiler's own placements
    /// included. A platform that cannot tell when a drag starts leaves it
    /// out.
    MoveStarted {
        /// The window.
        window: WindowId,
    },

    /// The user finished moving or resizing a window by dragging it.
    ///
    /// Reported once, when the drag ends. Never reported for moves that
    /// programs make, the tiler's own placements included, since the
    /// tiler retiles on it.
    Moved {
        /// The window.
        window: WindowId,
    },

    /// A window was minimized, by the user, its application or the tiler.
    Minimized {
        /// The window.
        window: WindowId,
    },

    /// A minimized window was shown again.
    ///
    /// May arrive after the window was already reported focused.
    Restored {
        /// The window.
        window: WindowId,
    },

    /// A window's title changed.
    ///
    /// Comes in bursts. Also reported for windows that were never
    /// reported created, which is how windows that only become
    /// recognizable once they have a title are found.
    TitleChanged {
        /// The window.
        window: WindowId,
    },

    /// A window's position or size changed, for any reason.
    ///
    /// That includes maximizing and restoring, which report no
    /// [`Moved`](Self::Moved), animations and the tiler's own placements.
    /// Comes in bursts. A platform that cannot report every step must at
    /// least report where the window ended up.
    LocationChanged {
        /// The window.
        window: WindowId,
    },

    /// Monitors were connected, disconnected, rearranged or changed
    /// resolution.
    ///
    /// Comes in bursts until the new configuration settles.
    DisplayChanged,

    /// The usable area of a monitor changed: a taskbar, dock or panel was
    /// shown, hidden, moved or resized.
    WorkAreaChanged,
}

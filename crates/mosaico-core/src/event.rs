//! What happens to windows and displays, as the tiler hears of it.
//!
//! An [`EventSource`](crate::event::EventSource) reports what the
//! platform observes, and
//! [`FocusFollowsMouse`](crate::event::FocusFollowsMouse) reports where
//! the pointer comes to rest.

mod drag;
mod hover;
mod naming;
mod rest;
mod source;
mod window_event;

pub use drag::{DragStart, Edges, Grab};
pub use hover::{FocusFollowsMouse, MousePointer, POLL_INTERVAL, REST_POLLS};
pub use source::{EventSource, EventWatch};
pub use window_event::WindowEvent;

//! Moving windows through the frames of their transitions.
//!
//! [`FrameControl`] is what a platform does with the frames.
//! [`FrameTable`] keeps what is known of the frames posted to each window
//! and decides, the same way for every platform, whether the next one
//! goes, and whether a window's other changes have to be posted behind
//! them: a platform only reads its windows and posts their moves, through
//! a [`FrameProbe`].
//!
//! [`FrameControl`]: crate::frames::FrameControl
//! [`FrameTable`]: crate::frames::FrameTable
//! [`FrameProbe`]: crate::frames::FrameProbe

mod control;
mod frame;
mod posted;
mod probe;
mod table;

pub use control::FrameControl;
pub use frame::{Frame, FrameOutcome};
pub use probe::FrameProbe;
pub use table::{ASYNC_GRACE, FrameTable, PENDING_LIMIT};

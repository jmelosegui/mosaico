//! The bar API: the public model of what a bar shows of mosaico, and the
//! differences between two of its states.
//!
//! Every bar reads mosaico through these types, the built-in one included,
//! so a field a bar needs is added here first. They are shaped as the wire
//! is and kept apart from the tiler's own types, so the tiler can change
//! without changing what a bar reads. Within an [`API_VERSION`], changes
//! only add: a bar ignores fields it does not know, and a field it misses
//! reads as its default.

mod diff;
mod state;

pub use diff::{Removed, Update};
pub use state::{Daemon, Focus, FocusedWindow, Monitor, Rect, State, Theme, Workspace};

/// The version of the bar API this build speaks. A change that would make
/// a bar misread a message is a new version.
pub const API_VERSION: u32 = 1;

/// The longest focused window title the API carries, in characters.
pub const MAX_TITLE_CHARS: usize = 1000;

#[cfg(test)]
#[path = "tests/diff_tests.rs"]
mod diff_tests;

#[cfg(test)]
#[path = "tests/examples_tests.rs"]
mod examples_tests;

#[cfg(test)]
#[path = "tests/arbitrary.rs"]
mod arbitrary;

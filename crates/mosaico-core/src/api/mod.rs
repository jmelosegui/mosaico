//! The bar API: the public model of what a bar shows of mosaico, the
//! differences between two of its states, and the sessions that carry
//! them to bars.
//!
//! Every bar reads mosaico through these types, the built-in one included,
//! so a field a bar needs is added here first. They are shaped as the wire
//! is and kept apart from the tiler's own types, so the tiler can change
//! without changing what a bar reads. Within an [`API_VERSION`], changes
//! only add: a bar ignores fields it does not know, and a field it misses
//! reads as its default.

pub mod client;
mod connection;
mod diff;
mod hub;
mod loop_request;
mod messages;
mod outbox;
mod protocol;
mod session;
mod state;
mod topics;
mod writer;

pub use connection::{ApiConnection, ApiListener, ApiReader, ApiWriter, Limits, Stopper};
pub use diff::{Removed, Update};
pub use hub::{Hub, ToLoop};
pub use loop_request::{LoopRequest, Reply};
pub use messages::{Ack, ApiError, Bye, DaemonMessage, ErrorCode, Snapshot, Welcome};
pub use protocol::{
    ClientMessage, Hello, Refusal, Request, Run, SUPPORTED_APIS, Subscribe, negotiate,
};
pub use session::serve_api;
pub use state::{Daemon, Focus, FocusedWindow, Monitor, Rect, State, Theme, Workspace};
pub use topics::{Topic, Topics};

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

#[cfg(test)]
#[path = "tests/protocol_tests.rs"]
mod protocol_tests;

#[cfg(test)]
#[path = "tests/fake_pipe.rs"]
pub(crate) mod fake_pipe;

#[cfg(test)]
#[path = "tests/fake_client.rs"]
pub(crate) mod fake_client;

#[cfg(test)]
#[path = "tests/session_world.rs"]
pub(crate) mod session_world;

#[cfg(test)]
#[path = "tests/session_tests.rs"]
mod session_tests;

#[cfg(test)]
#[path = "tests/session_limits_tests.rs"]
mod session_limits_tests;

#[cfg(test)]
#[path = "tests/metrics_tests.rs"]
mod metrics_tests;

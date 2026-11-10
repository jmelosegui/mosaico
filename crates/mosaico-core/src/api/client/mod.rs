//! The client side of the bar API: what `mosaico query`, `subscribe` and
//! `run` do, apart from reading their arguments.
//!
//! It keeps the state as JSON, not as [`State`](super::State), so it passes
//! on the fields a newer daemon adds, and it reaches the daemon through
//! [`ApiClient`], so it is tested against the real sessions over memory.

mod connect;
mod field;
mod json_state;
mod link;
mod monitor_ref;
mod oneshot;
mod output;
mod queue;
mod space;
mod subscribe;
mod subscription;

pub use connect::{ApiClient, ClientConnection};
pub use field::{FieldPath, field_text};
pub use json_state::JsonState;
pub use link::Outcome;
pub use monitor_ref::MonitorRef;
pub use oneshot::{Query, query, run};
pub use output::Output;
pub use space::{Hold, OWN_IDS};
pub use subscribe::subscribe;
pub use subscription::{OpenInput, Streams, Subscription};

#[cfg(test)]
#[path = "../tests/client_state_tests.rs"]
mod state_tests;

#[cfg(test)]
#[path = "../tests/client_tests.rs"]
mod client_tests;

#[cfg(test)]
#[path = "../tests/client_subscribe_tests.rs"]
mod subscribe_tests;

#[cfg(test)]
#[path = "../tests/client_space_tests.rs"]
mod space_tests;

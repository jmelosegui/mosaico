//! Finding newer releases of mosaico.
//!
//! Releases are published on GitHub. Looking them up goes through an
//! [`HttpClient`](crate::http::HttpClient), so it is tested without a
//! network.

mod release;

pub use release::{is_newer_tag, latest_tag, newer_release};

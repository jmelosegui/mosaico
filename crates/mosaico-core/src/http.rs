//! Fetching files from web servers.

use std::fmt;
use std::time::Duration;

use crate::{Error, Result};

/// Fetching files from web servers over HTTPS.
///
/// Looking up releases, downloading them and fetching the community
/// rules go through this, so all of that can be tested without a
/// network.
///
/// # Contract for platform implementations
///
/// - `get_bytes` sends a GET request for `path` to `host` over HTTPS and
///   returns the body once all of it has arrived. It blocks the calling
///   thread until then.
/// - It follows redirects, also to other hosts: release downloads are
///   redirected to the host that stores the files.
/// - An answer with a status outside 200-299 is
///   [`Error::http_status`], never a body.
/// - `timeout` bounds each step on its own: resolving the host,
///   connecting, sending the request and every wait for more of the
///   answer. A download that keeps receiving data is not cut off.
/// - A host that cannot be reached, a step that takes longer than
///   `timeout` and a connection that breaks are
///   [`Error::http_request`], naming the step that failed.
/// - Both name the request by its [`url`], so failures read the same on
///   every platform.
/// - Each request tells the server it comes from mosaico, and which
///   version, since some servers refuse anonymous requests.
pub trait HttpClient {
    /// Fetches `path` from `host` and returns the body.
    fn get_bytes(&self, host: &str, path: &str, timeout: Duration) -> Result<Vec<u8>>;

    /// Fetches `path` from `host` and returns the body as text.
    ///
    /// A body that is not UTF-8 is an [`Error::Network`].
    fn get_text(&self, host: &str, path: &str, timeout: Duration) -> Result<String> {
        let body = self.get_bytes(host, path, timeout)?;
        String::from_utf8(body).map_err(|e| {
            Error::Network(format!("{} did not answer with text: {e}", url(host, path)))
        })
    }
}

/// The URL of `path` on `host`, as requests and their errors name it.
pub fn url(host: &str, path: &str) -> String {
    format!("https://{host}{path}")
}

/// How a request fails, worded the same on every platform.
impl Error {
    /// The server answered the request for `url` with `status`, which is
    /// outside 200-299.
    pub fn http_status(url: &str, status: u16) -> Self {
        Self::Network(format!("{url} answered with HTTP status {status}"))
    }

    /// The request for `url` failed while trying to `step`, such as "send
    /// the request", for the reason `cause` gives.
    pub fn http_request(url: &str, step: &str, cause: impl fmt::Display) -> Self {
        Self::Network(format!("{url}: could not {step}: {cause}"))
    }
}

#[cfg(test)]
#[path = "http_fake.rs"]
pub(crate) mod fake;

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;

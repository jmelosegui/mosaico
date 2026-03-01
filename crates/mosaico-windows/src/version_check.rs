//! Checks GitHub for a newer release of mosaico.
//!
//! Thin HTTP wrapper around the platform-agnostic comparison logic
//! in [`mosaico_core::version`].

use mosaico_core::version;

use crate::http;

const GITHUB_HOST: &str = "api.github.com";
const RELEASES_PATH: &str = "/repos/jmelosegui/mosaico/releases/latest";
const TIMEOUT_MS: i32 = 5000;

/// Returns the latest release tag from GitHub (e.g. `"v0.2.0"`),
/// or `None` on network error or missing tag.
pub fn fetch_latest_tag() -> Option<String> {
    let body = http::get(GITHUB_HOST, RELEASES_PATH, TIMEOUT_MS).ok()?;
    version::extract_tag_name(&body)
}

/// Returns the remote version (e.g. `"v0.2.1"`) if it is newer than
/// the running binary, or `None` otherwise.
pub fn check_for_update() -> Option<String> {
    let body = http::get(GITHUB_HOST, RELEASES_PATH, TIMEOUT_MS).ok()?;
    version::check_for_update(&body, env!("CARGO_PKG_VERSION"))
}

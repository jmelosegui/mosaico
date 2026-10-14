//! Looking up the latest release on GitHub.

use std::time::Duration;

use crate::http::HttpClient;
use crate::version;
use crate::{Error, Result};

/// The host of the GitHub API.
const API_HOST: &str = "api.github.com";

/// The latest release of mosaico in the GitHub API.
const LATEST_RELEASE: &str = "/repos/jmelosegui/mosaico/releases/latest";

/// How long each step of the lookup may take.
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);

/// The tag of the latest release, such as `v0.2.0`.
///
/// # Errors
///
/// Fails with [`Error::Network`] when GitHub cannot be reached or its
/// answer names no release tag.
pub fn latest_tag(http: &dyn HttpClient) -> Result<String> {
    let body = http.get_text(API_HOST, LATEST_RELEASE, LOOKUP_TIMEOUT)?;
    version::extract_tag_name(&body).ok_or_else(|| {
        Error::Network(format!(
            "https://{API_HOST}{LATEST_RELEASE} named no release tag"
        ))
    })
}

/// The tag of the latest release if it is newer than `local`, the
/// version that is running, such as `0.1.0`.
///
/// # Errors
///
/// Fails like [`latest_tag`], so a lookup that failed is not taken for
/// being up to date.
pub fn newer_release(http: &dyn HttpClient, local: &str) -> Result<Option<String>> {
    let tag = latest_tag(http)?;
    Ok(is_newer_tag(&tag, local).then_some(tag))
}

/// Whether the release tagged `tag`, a version with or without a `v` in
/// front, is newer than `local`.
pub fn is_newer_tag(tag: &str, local: &str) -> bool {
    version::is_newer(tag.strip_prefix('v').unwrap_or(tag), local)
}

#[cfg(test)]
#[path = "tests/release_tests.rs"]
mod tests;

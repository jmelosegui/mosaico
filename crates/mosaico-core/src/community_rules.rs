//! Downloads community rules from the mosaico-rules GitHub repository.
//!
//! On daemon startup a background thread fetches the rules file for its
//! platform from `raw.githubusercontent.com`, validates it as TOML, and
//! caches it as the instance's `rules.toml`. If the download fails or
//! the content is invalid, the existing cached file is kept.

use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

use crate::config::{self, WindowRule};
use crate::http::HttpClient;
use crate::{Error, Result};

/// GitHub raw content host (no API rate limit for public repos).
const RULES_HOST: &str = "raw.githubusercontent.com";

/// How long each step of the download may take.
const TIMEOUT: Duration = Duration::from_secs(5);

/// Downloads the community rules of `rule_set`, validates them and
/// writes them to `cache`.
///
/// `rule_set` is the folder of the mosaico-rules repository that holds
/// the rules for this platform. `cache` is only written once the
/// download has turned out to be valid rules, and then replaced in one
/// step, so a failure leaves it as it was. Returns the downloaded rules;
/// the caller merges them with the user's.
///
/// # Errors
///
/// Fails with [`Error::Network`] when the download fails, with
/// [`Error::Config`] when it is not a rules file or has no rules, and
/// with [`Error::Io`] when `cache` cannot be written.
pub fn download(http: &dyn HttpClient, rule_set: &str, cache: &Path) -> Result<Vec<WindowRule>> {
    let path = format!("/jmelosegui/mosaico-rules/main/{rule_set}/rules.toml");
    let body = http
        .get_text(RULES_HOST, &path, TIMEOUT)
        .inspect_err(|e| crate::log_info!("Community rules download failed: {e}"))?;

    // Validate before writing -- don't overwrite with garbage.
    let rules = match config::validate_rules(&body) {
        Ok(r) if r.is_empty() => {
            crate::log_info!("Community rules file has no rules, skipping");
            return Err(Error::config(None, "the community rules file has no rules"));
        }
        Ok(r) => r,
        Err(e) => {
            crate::log_info!("Community rules invalid TOML: {e}");
            return Err(Error::config(
                None,
                format!("the community rules are not valid: {e}"),
            ));
        }
    };

    replace(cache, &body)
        .inspect_err(|e| crate::log_info!("Failed to cache community rules: {e}"))?;

    crate::log_info!("Downloaded {} community rules", rules.len());
    Ok(rules)
}

/// Replaces the file at `cache` with one holding `content`.
///
/// The content is written to a file beside it, which is then renamed
/// over it. Writing into `cache` itself would empty it first, and leave
/// it empty if the write then failed.
fn replace(cache: &Path, content: &str) -> io::Result<()> {
    let mut name = cache.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    let partial = cache.with_file_name(name);
    let replaced = fs::write(&partial, content).and_then(|()| fs::rename(&partial, cache));
    if replaced.is_err() {
        let _ = fs::remove_file(&partial);
    }
    replaced
}

#[cfg(test)]
#[path = "community_rules_tests.rs"]
mod tests;

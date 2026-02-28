//! Downloads community rules from the mosaico-rules GitHub repository.
//!
//! On daemon startup a background thread fetches the platform-specific
//! rules file from `raw.githubusercontent.com`, validates it as TOML,
//! and caches it at `~/.config/mosaico/rules.toml`. If the download
//! fails or the content is invalid, the existing cached file is kept.

use mosaico_core::config::{self, WindowRule};

/// GitHub raw content host (no API rate limit for public repos).
const RULES_HOST: &str = "raw.githubusercontent.com";

/// Path to the Windows community rules file.
const RULES_PATH: &str = "/jmelosegui/mosaico-rules/main/windows/rules.toml";

/// Download timeout in milliseconds.
const TIMEOUT_MS: i32 = 5000;

/// Downloads community rules, validates and caches them.
///
/// Returns the merged rule set (user + community) on success so the
/// caller can send a reload to the daemon. Returns `None` on any
/// failure (network, parse, IO) — the caller should continue with
/// whatever rules are already cached.
pub fn download() -> Option<Vec<WindowRule>> {
    let body = match crate::http::get(RULES_HOST, RULES_PATH, TIMEOUT_MS) {
        Ok(b) => b,
        Err(e) => {
            mosaico_core::log_info!("Community rules download failed: {e}");
            return None;
        }
    };

    // Validate before writing — don't overwrite with garbage.
    let rules = match config::validate_rules(&body) {
        Ok(r) if r.is_empty() => {
            mosaico_core::log_info!("Community rules file has no rules, skipping");
            return None;
        }
        Ok(r) => r,
        Err(e) => {
            mosaico_core::log_info!("Community rules invalid TOML: {e}");
            return None;
        }
    };

    let path = config::rules_path()?;
    if let Err(e) = std::fs::write(&path, &body) {
        mosaico_core::log_info!("Failed to cache community rules: {e}");
        return None;
    }

    mosaico_core::log_info!("Downloaded {} community rules", rules.len());
    Some(config::load_merged_rules())
}

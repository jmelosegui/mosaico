//! Checks GitHub for a newer release of mosaico.
//!
//! Returns the remote version string when an update is available,
//! or `None` on same version, network errors, or timeouts.

const GITHUB_HOST: &str = "api.github.com";
const RELEASES_PATH: &str = "/repos/jmelosegui/mosaico/releases/latest";
const TIMEOUT_MS: i32 = 3000;

/// Returns the remote version (e.g. `"v0.2.0"`) if it is newer than
/// the running binary, or `None` otherwise.
pub fn check_for_update() -> Option<String> {
    let body = mosaico_windows::http::get(GITHUB_HOST, RELEASES_PATH, TIMEOUT_MS).ok()?;
    let tag = extract_tag_name(&body)?;
    let remote = tag.strip_prefix('v').unwrap_or(&tag);
    let local = env!("CARGO_PKG_VERSION");

    if is_newer(remote, local) {
        Some(tag)
    } else {
        None
    }
}

/// Extracts the `tag_name` value from a GitHub releases JSON response.
///
/// Looks for `"tag_name":"..."` or `"tag_name" : "..."` and returns
/// the inner string value.  Avoids pulling in a JSON parser dependency.
fn extract_tag_name(json: &str) -> Option<String> {
    let key = "\"tag_name\"";
    let start = json.find(key)? + key.len();
    let rest = &json[start..];
    let quote_start = rest.find('"')? + 1;
    let inner = &rest[quote_start..];
    let quote_end = inner.find('"')?;
    Some(inner[..quote_end].to_string())
}

/// Returns `true` when `remote` is a higher semver than `local`.
///
/// Compares each dot-separated part as an integer. Non-numeric or
/// malformed versions return `false` (treat as "not newer").
fn is_newer(remote: &str, local: &str) -> bool {
    let r: Vec<u32> = remote.split('.').filter_map(|p| p.parse().ok()).collect();
    let l: Vec<u32> = local.split('.').filter_map(|p| p.parse().ok()).collect();
    if r.len() != 3 || l.len() != 3 {
        return false;
    }
    (r[0], r[1], r[2]) > (l[0], l[1], l[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_major() {
        assert!(is_newer("1.0.0", "0.9.9"));
    }

    #[test]
    fn newer_minor() {
        assert!(is_newer("0.2.0", "0.1.9"));
    }

    #[test]
    fn newer_patch() {
        assert!(is_newer("0.1.1", "0.1.0"));
    }

    #[test]
    fn same_version() {
        assert!(!is_newer("0.1.0", "0.1.0"));
    }

    #[test]
    fn older_version() {
        assert!(!is_newer("0.1.0", "0.2.0"));
    }

    #[test]
    fn malformed_returns_false() {
        assert!(!is_newer("abc", "0.1.0"));
        assert!(!is_newer("0.1", "0.1.0"));
    }

    #[test]
    fn extract_tag_from_json() {
        let json = r#"{"tag_name":"v0.2.0","name":"Release 0.2.0"}"#;
        assert_eq!(extract_tag_name(json), Some("v0.2.0".to_string()));
    }

    #[test]
    fn extract_tag_with_spaces() {
        let json = r#"{ "tag_name" : "v1.0.0" }"#;
        assert_eq!(extract_tag_name(json), Some("v1.0.0".to_string()));
    }

    #[test]
    fn extract_tag_missing() {
        assert_eq!(extract_tag_name(r#"{"name":"foo"}"#), None);
    }
}

//! Platform-agnostic version comparison utilities.
//!
//! Parses GitHub release JSON and compares semver strings.
//! The actual HTTP fetch is platform-specific and lives in the
//! platform crate (e.g. `mosaico-windows::version_check`).

/// Returns `true` when `remote` is a higher semver than `local`.
///
/// Compares each dot-separated part as an integer. Non-numeric or
/// malformed versions return `false` (treat as "not newer").
pub fn is_newer(remote: &str, local: &str) -> bool {
    let r: Vec<u32> = remote.split('.').filter_map(|p| p.parse().ok()).collect();
    let l: Vec<u32> = local.split('.').filter_map(|p| p.parse().ok()).collect();
    if r.len() != 3 || l.len() != 3 {
        return false;
    }
    (r[0], r[1], r[2]) > (l[0], l[1], l[2])
}

/// Extracts the `tag_name` value from a GitHub releases JSON response.
///
/// Looks for `"tag_name":"..."` or `"tag_name" : "..."` and returns
/// the inner string value.  Avoids pulling in a JSON parser dependency.
pub fn extract_tag_name(json: &str) -> Option<String> {
    let key = "\"tag_name\"";
    let start = json.find(key)? + key.len();
    let rest = &json[start..];
    let quote_start = rest.find('"')? + 1;
    let inner = &rest[quote_start..];
    let quote_end = inner.find('"')?;
    Some(inner[..quote_end].to_string())
}

/// Returns the remote tag if it is newer than `local_version`.
///
/// Parses a GitHub releases JSON body, extracts the tag, and compares
/// it against the given local version string.
pub fn check_for_update(json: &str, local_version: &str) -> Option<String> {
    let tag = extract_tag_name(json)?;
    let remote = tag.strip_prefix('v').unwrap_or(&tag);
    if is_newer(remote, local_version) {
        Some(tag)
    } else {
        None
    }
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

    #[test]
    fn check_finds_update() {
        let json = r#"{"tag_name":"v0.2.0"}"#;
        assert_eq!(check_for_update(json, "0.1.0"), Some("v0.2.0".into()));
    }

    #[test]
    fn check_no_update_when_same() {
        let json = r#"{"tag_name":"v0.1.0"}"#;
        assert_eq!(check_for_update(json, "0.1.0"), None);
    }

    #[test]
    fn check_no_update_when_local_is_newer() {
        let json = r#"{"tag_name":"v0.1.0"}"#;
        assert_eq!(check_for_update(json, "0.2.0"), None);
    }
}

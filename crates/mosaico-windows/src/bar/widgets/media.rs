//! Media widget — displays the currently playing track from system
//! media sources using the Windows GSMTC API.

use super::BarState;

/// Returns the widget display text, truncated to `max_length`.
pub fn text(state: &BarState, max_length: usize) -> String {
    if state.media_text.is_empty() || max_length == 0 {
        return String::new();
    }
    truncate(&state.media_text, max_length)
}

/// Queries the current media session and returns "Artist - Title".
///
/// Returns an empty string if nothing is playing or the API is
/// unavailable. This is called on each bar tick (1 second).
///
/// **Note:** The GSMTC `RequestAsync().get()` call blocks forever in
/// our daemon process (no GUI message pump). This stub returns empty
/// until a non-blocking implementation is added.
pub fn query_media() -> String {
    // TODO: implement non-blocking GSMTC query
    String::new()
}

/// Truncates a string to `max` characters, adding "..." if needed.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
    format!("{truncated}...")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_exact_length_unchanged() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_long_string_adds_ellipsis() {
        assert_eq!(truncate("hello world", 8), "hello...");
    }

    #[test]
    fn text_returns_empty_when_no_media() {
        let state = BarState::default();
        assert!(text(&state, 40).is_empty());
    }

    #[test]
    fn text_truncates_to_max_length() {
        let state = BarState {
            media_text: "Very Long Artist Name - Very Long Track Title".into(),
            ..Default::default()
        };
        let result = text(&state, 20);
        assert!(result.ends_with("..."));
        assert!(result.chars().count() <= 20);
    }
}

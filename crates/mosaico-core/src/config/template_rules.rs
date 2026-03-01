/// Generates the default `rules.toml` contents with explanatory comments.
///
/// This serves as the initial cache before community rules are
/// downloaded on the first daemon startup.
pub fn generate_rules() -> String {
    r#"# Community-maintained default rules for Mosaico
# Location: ~/.config/mosaico/rules.toml
#
# This file is downloaded automatically when the daemon starts.
# To override a rule, add an entry in ~/.config/mosaico/user-rules.toml
# rather than editing this file (it will be overwritten on next startup).
#
# To contribute: https://github.com/jmelosegui/mosaico-rules

# UWP ghost frames — invisible companion windows with no title.
[[rule]]
match_class = "ApplicationFrameWindow"
match_title = ""
manage = false

# GPG passphrase prompt — small modal dialog, should not be tiled.
[[rule]]
match_title = "pinentry"
manage = false
"#
    .to_string()
}

/// Generates the default `user-rules.toml` contents with explanatory comments.
///
/// This is used by `mosaico init` to create a starter user rules file
/// where users can add personal overrides that are never overwritten.
pub fn generate_user_rules() -> String {
    r#"# User-specific window rules for Mosaico
# Location: ~/.config/mosaico/user-rules.toml
#
# Rules in this file take priority over community defaults in rules.toml.
# Community rules are downloaded automatically — to contribute a rule
# that benefits everyone, please submit it to:
#
#   https://github.com/jmelosegui/mosaico-rules
#
# Only add rules here for personal preferences that don't apply to all
# users (e.g., tiling a specific app that most people would exclude).

# Example: force-tile a window that community rules exclude
# [[rule]]
# match_class = "MySpecialApp"
# manage = true

# Example: exclude a personal app
# [[rule]]
# match_title = "My Private Tool"
# manage = false
"#
    .to_string()
}

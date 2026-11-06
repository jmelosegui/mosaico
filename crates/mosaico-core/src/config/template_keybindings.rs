use super::keybindings_entries;
use super::keybindings_header;

/// Generates the default `keybindings.toml` contents with explanatory comments.
///
/// This is used by `mosaico init` to create a starter keybindings file.
pub fn generate_keybindings() -> String {
    let mut content = String::new();
    content.push_str(keybindings_header::header());
    content.push_str(&keybindings_entries::entries());
    content.push_str(LAUNCH_EXAMPLES);
    content
}

/// Launch bindings are left for the user to choose, so the template only
/// shows them, commented out.
const LAUNCH_EXAMPLES: &str = r#"
# Launch bindings: a key combination that starts an application.
#
#   command   program to start: a name like "msedge.exe", a full path, or a
#             URI like "ms-settings:"
#   args      arguments, one string each (optional)
#   focus     true focuses an open window of the application instead of
#             starting another one (optional, default false)
#   match_path, match_class, match_title
#             which windows count as the application's, as in window rules
#             (optional; by default, windows of the program named by command)
#
# [[launch]]
# command = "wt.exe"
# key = "Enter"
# modifiers = ["alt"]
#
# [[launch]]
# command = "msedge.exe"
# key = "B"
# modifiers = ["alt", "shift"]
# focus = true
"#;

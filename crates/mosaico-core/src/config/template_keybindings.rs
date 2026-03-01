use super::keybindings_entries;
use super::keybindings_header;

/// Generates the default `keybindings.toml` contents with explanatory comments.
///
/// This is used by `mosaico init` to create a starter keybindings file.
pub fn generate_keybindings() -> String {
    let mut content = String::new();
    content.push_str(keybindings_header::header());
    content.push_str(&keybindings_entries::entries());
    content
}

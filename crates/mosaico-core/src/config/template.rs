#[path = "keybindings_entries.rs"]
mod keybindings_entries;
#[path = "keybindings_focus_move.rs"]
mod keybindings_focus_move;
#[path = "keybindings_header.rs"]
mod keybindings_header;
#[path = "keybindings_workspaces.rs"]
mod keybindings_workspaces;
#[path = "template_bar.rs"]
mod template_bar;
#[path = "template_config.rs"]
mod template_config;
#[path = "template_keybindings.rs"]
mod template_keybindings;
#[path = "template_rules.rs"]
mod template_rules;

pub use template_bar::generate_bar;
pub use template_config::generate_config;
pub use template_keybindings::generate_keybindings;
pub use template_rules::{generate_rules, generate_user_rules};

#[cfg(test)]
#[path = "template_tests.rs"]
mod tests;

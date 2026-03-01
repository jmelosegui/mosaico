use super::keybindings_focus_move;
use super::keybindings_workspaces;

pub(super) fn entries() -> String {
    let mut content = String::new();
    content.push_str(keybindings_focus_move::entries());
    content.push_str(keybindings_workspaces::entries());
    content
}

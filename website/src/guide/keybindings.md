# Keyboard Bindings

Mosaico registers global system hotkeys that work regardless of which
application has focus. Keybindings are configured in
`~/.config/mosaico/keybindings.toml`.

## Default Keybindings

The defaults use vim-style spatial navigation (H=left, J=down, K=up,
L=right):

| Shortcut | Action |
|----------|--------|
| `Alt + H` | Focus left |
| `Alt + J` | Focus down |
| `Alt + K` | Focus up |
| `Alt + L` | Focus right |
| `Alt + Shift + H` | Move left |
| `Alt + Shift + J` | Move down |
| `Alt + Shift + K` | Move up |
| `Alt + Shift + L` | Move right |
| `Alt + Shift + R` | Retile |
| `Alt + T` | Toggle monocle |
| `Alt + Q` | Close focused window |
| `Alt + 1` -- `Alt + 8` | Switch to workspace 1-8 |
| `Alt + Shift + 1` -- `Alt + Shift + 8` | Send window to workspace 1-8 |

## Configuration Format

Each binding is a `[[keybinding]]` entry with three fields:

```toml
[[keybinding]]
action = "focus-left"
key = "H"
modifiers = ["alt"]

[[keybinding]]
action = "move-left"
key = "H"
modifiers = ["alt", "shift"]
```

### Actions

| Action | Description |
|--------|-------------|
| `focus-left` | Focus the window to the left |
| `focus-right` | Focus the window to the right |
| `focus-up` | Focus the window above |
| `focus-down` | Focus the window below |
| `move-left` | Move window left (swap or cross-monitor) |
| `move-right` | Move window right |
| `move-up` | Move window up (swap on same monitor) |
| `move-down` | Move window down |
| `retile` | Re-apply the tiling layout |
| `toggle-monocle` | Toggle monocle (full-screen) mode |
| `close-focused` | Close the focused window |
| `goto-workspace-N` | Switch to workspace N (1-8) |
| `send-to-workspace-N` | Send focused window to workspace N (1-8) |

### Modifiers

| Modifier | Key |
|----------|-----|
| `alt` | Alt key |
| `shift` | Shift key |
| `ctrl` | Control key |
| `win` | Windows key |

### Supported Keys

| Key Type | Examples |
|----------|----------|
| Letters | `A` -- `Z` (case-insensitive) |
| Digits | `0` -- `9` |
| Function keys | `F1` -- `F12` |
| Named keys | `Enter`, `Tab`, `Escape`, `Space` |
| Arrow keys | `Left`, `Right`, `Up`, `Down` |
| Punctuation | `Minus`, `Plus`, `Comma`, `Period` |

Unknown key names are skipped with a log message.

## Reloading

Keybinding changes are **not** hot-reloaded. You must restart the daemon
for changes to take effect:

```sh
mosaico stop && mosaico start
```

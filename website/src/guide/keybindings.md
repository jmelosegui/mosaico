# Keyboard Bindings

Mosaico registers global system hotkeys that work regardless of which
application has focus. Keybindings are configured in
`~/.config/mosaico/keybindings.toml`.

## Default Keybindings

The defaults use vim-style spatial navigation (H=left, J=down, K=up,
L=right). For a printable one-page reference, see the
[Cheatsheet](cheatsheet.md).

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
| `Alt + N` | Cycle layout |
| `Alt + M` | Minimize focused window |
| `Alt + Q` | Close focused window |
| `Alt + 1` .. `Alt + 8` | Switch to workspace 1-8 |
| `Alt + Shift + 1` .. `Alt + Shift + 8` | Send window to workspace 1-8 |
| `Alt + Shift + P` | Toggle pause / resume |

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
| `retile` | Re-apply the tiling layout, with the splits you moved back at the configured ratio, or an equal share in a stack, and the windows you floated tiled again |
| `toggle-monocle` | Toggle monocle (full-screen) mode |
| `toggle-float` | Float the focused window above the tiles, or tile it again |
| `cycle-float` | Focus the next floating window of the workspace on screen |
| `grow-width` / `shrink-width` | Widen or narrow the focused floating window by one step |
| `grow-height` / `shrink-height` | Heighten or shorten the focused floating window by one step |
| `cycle-layout` | Cycle to the next tiling layout (BSP -> VStack -> 3Col) |
| `close-focused` | Close the focused window |
| `minimize-focused` | Minimize the focused window |
| `goto-workspace-N` | Switch to workspace N (1-8) |
| `goto-workspace-next` | Switch to the workspace after the one on screen |
| `goto-workspace-prev` | Switch to the workspace before the one on screen |
| `send-to-workspace-N` | Send focused window to workspace N (1-8) |
| `toggle-pause` | Toggle hotkey pause on/off |

`goto-workspace-next` and `goto-workspace-prev` have no default shortcut.
Past the last workspace they stop, or go round to the first with
`[workspaces] wrap = true` in `config.toml`. For example:

```toml
[[keybinding]]
action = "goto-workspace-next"
key = "Right"
modifiers = ["alt", "ctrl"]
```

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
| Function keys | `F1` -- `F24` |
| Named keys | `Enter`, `Tab`, `Escape`, `Space` |
| Arrow keys | `Left`, `Right`, `Up`, `Down` |
| Punctuation | `Minus`, `Plus`, `Comma`, `Period` |

Unknown key names are skipped with a log message.

## Pause / Unpause

If another application uses the same global shortcuts as mosaico, you can
temporarily release mosaico's hotkeys so that application can receive them:

```toml
[[keybinding]]
action = "toggle-pause"
key = "P"
modifiers = ["alt", "shift"]
```

Press `Alt+Shift+P` to pause — the status bar shows a red **PAUSED** indicator.
Press it again, or click the indicator, to resume. You can also use the CLI:

```sh
mosaico pause
mosaico unpause
```

The `toggle-pause` hotkey stays registered while paused so you can always
resume from the keyboard without opening a terminal.

## Auto-merge

On each daemon start, any default keybinding actions not present in your
`keybindings.toml` are automatically appended to the file. This ensures you
pick up new shortcuts added in future versions without overwriting any of your
customizations. Actions you've already bound (even to different keys) are
never touched.

## Reloading

Keybinding changes are **not** hot-reloaded. You must restart the daemon
for changes to take effect:

```sh
mosaico stop && mosaico start
```

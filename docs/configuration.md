# Configuration System & Window Rules

Mosaico uses five TOML configuration files stored in `~/.config/mosaico/`.
All settings have sensible defaults, so configuration files are optional.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/config/mod.rs` | `Config`, `Config::validate()` |
| `crates/mosaico-core/src/config/types.rs` | `LayoutConfig`, `BorderConfig`, `BorderColors`, `CornerStyle` |
| `crates/mosaico-core/src/config/workspaces.rs` | `WorkspacesConfig`, `WorkspaceMode`, the `[workspaces]` section |
| `crates/mosaico-core/src/config/mouse.rs` | `MouseConfig`, the `[mouse]` section: the pointer and focus, and dragging windows |
| `crates/mosaico-core/src/config/rules.rs` | `WindowRule`, `should_manage()`, `matching_rule()`, `default_rules()` |
| `crates/mosaico-core/src/config/hiding_behaviour.rs` | `HidingBehaviour`, how windows are hidden on a workspace switch |
| `crates/mosaico-core/src/config/animations.rs` | `AnimationConfig`, the `[animations]` section: window transitions |
| `crates/mosaico-core/src/motion/easing.rs`, `easing_names.rs`, `bezier.rs` | `Easing`, the curves transitions follow, and the names `config.toml` writes them with |
| `crates/mosaico-core/src/config/store/` | `ConfigStore`: the paths of the config files (`mod.rs`), loading them (`load.rs`), each migration (`migrate*.rs`) and each merge of new defaults (`merge_*.rs`) |
| `crates/mosaico-core/src/config/reload/` | `ConfigReloader`, `ConfigReload`: which files hot-reload and how (`mod.rs`); `ConfigWatch`: opening the watcher after the files are recorded, and the watch loop (`run.rs`) |
| `crates/mosaico-core/src/config/keybinding.rs` | `Keybinding`, `Modifier`, `defaults()`, `Keybinding::combo()` |
| `crates/mosaico-core/src/config/bar.rs` | `BarConfig`, `BarColors`, `WidgetConfig` |
| `crates/mosaico-core/src/config/bar_mouse.rs` | `BarMouseConfig`, `BarScroll`, `ClickBinding`, the `[mouse]` section of `bar.toml` and its `[[mouse.click]]` entries |
| `crates/mosaico-core/src/config/widget_kind.rs` | `WidgetKind`, a widget's type by name, and `WidgetConfig::kind()` |
| `crates/mosaico-core/src/log.rs` | `LogConfig`, the `[logging]` section, and the logger |
| `crates/mosaico-core/src/config/theme_config.rs` | `ThemeConfig`, the `[theme]` section |
| `crates/mosaico-core/src/config/theme.rs` | `Theme` enum |
| `crates/mosaico-core/src/config/palette/` | The color palettes of each theme (hardcoded named color data) |
| `crates/mosaico-core/src/config/template.rs` | Template generators for `mosaico init`, `config.toml`'s in `template_config.rs` |

### Key Types

- `Config` -- top-level config: `theme: ThemeConfig`, `layout: LayoutConfig`,
  `workspaces: WorkspacesConfig`, `borders: BorderConfig`,
  `mouse: MouseConfig`, `animations: AnimationConfig`, `logging: LogConfig`
- `LayoutConfig` -- `default: LayoutKind` (default `bsp`), `gap: i32` (default
  8), `ratio: f64` (default 0.5, where every split is until the user moves
  it), `hiding: HidingBehaviour` (default `Cloak`)
- `WorkspacesConfig` -- `mode: WorkspaceMode` (default `per-monitor`),
  `wrap: bool` (default false: stepping through the workspaces stops at
  either end), `layouts: HashMap<u8, LayoutKind>` (per-workspace layout
  overrides)
- `MouseConfig` -- `follows_focus: bool` (default true),
  `focus_follows_mouse: bool` (default false), `drag_to_swap: bool`
  (default true), `resize_splits: bool` (default true)
- `HidingBehaviour` (enum) -- `Cloak`, `Hide`, `Minimize`, each described by
  what the user sees. One the platform does not have falls back to one it
  has (see [workspaces.md](workspaces.md#hiding-behaviour))
- `BorderConfig` -- `width: i32` (default 4), `corner_style: CornerStyle`,
  `colors: BorderColors`
- `BorderColors` -- `focused: String`, `monocle: String`, and
  `unfocused: String`. Empty strings resolve from the active theme
  (Mocha defaults: `"#89b4fa"` blue, `"#a6e3a1"` green, `"#6c7086"`
  muted gray). The literal `"none"` on `unfocused` disables borders
  for unfocused windows entirely; `BorderColors::unfocused_enabled()`
  reports the toggle state
- `AnimationConfig` -- `enabled: bool` (default true), `duration: i32` in
  milliseconds (default 180), `easing: Easing` (default `ease-out-cubic`),
  `follow_system: bool` (default true: no transitions while the system's
  settings turn animations off, or in a remote session). `is_on()` says
  whether transitions are on: enabled, with a duration above zero. `spec()`
  gives the duration and easing a transition follows (`motion::MotionSpec`).
  What animates, and why the rest snaps, is in [animations.md](animations.md)
- `Easing` (in `mosaico_core::motion`) -- `linear`, `ease-out-quad`,
  `ease-out-cubic`, `ease-in-out-cubic`, `ease-out-expo`, or
  `cubic-bezier(x1, y1, x2, y2)` as CSS writes it, with all four numbers
  within 0 to 1 so that windows never overshoot. Written and stored by name
- `LogConfig` -- `enabled: bool` (default false), `level: String` (default
  `"info"`), `max_file_mb: u64` (default 10)
- `WindowRule` -- `match_class: Option<String>`, `match_title: Option<String>`,
  `match_path: Option<String>`, `manage: bool`
- `ThemeConfig` -- `name: String` (default `"catppuccin"`) and
  `flavor: String` (default `"mocha"`; empty stands for the default flavor
  of the family)
- `Theme` (enum) -- `Catppuccin(CatppuccinFlavor)`, `RosePine(RosePineFlavor)`,
  `TokyoNight(TokyoNightFlavor)`
- `BarConfig` -- `enabled: bool`, `height: i32`, `monitors: Vec<usize>`
  (empty for every monitor), `mouse: BarMouseConfig`,
  `left/right: Vec<WidgetConfig>`, `colors: BarColors`, and the font,
  padding and pill settings
- `BarMouseConfig` -- `enabled: bool` (default true: the bar takes clicks
  and the wheel), `scroll: BarScroll` (default `workspaces`, or `none`),
  `click: Vec<ClickBinding>` (default `ClickBinding::defaults()`: the layout
  widget's left and right buttons cycle the layout and toggle monocle, and
  the paused widget's left one toggles the pause)
- `ClickBinding` -- `widget`, `button` and `action: Option<String>`, a
  `[[mouse.click]]` entry as written, `None` for a key it leaves out, kept as
  text and checked by `WidgetClicks` (in `mosaico_core::bar`) as the bar
  takes the config; `action = "none"` (`ClickBinding::NONE`) removes a
  binding
- `BarColors` -- `background`, `foreground`, `accent`, the workspace,
  separator, widget and pill colors (support named colors)
- `KeybindingsFile` -- wrapper for TOML deserialization of `[[keybinding]]`
  arrays
- `RulesFile` -- wrapper for TOML deserialization of `[[rule]]` arrays

## Configuration Files

### `config.toml`

Controls layout, borders, the mouse, window transitions, logging, and theme:

```toml
[layout]
default = "bsp"  # Default layout algorithm: "bsp", "vertical-stack", "three-column"
gap = 8          # Pixel gap between windows (0-200)
ratio = 0.5      # BSP split ratio (0.1-0.9)
hiding = "cloak" # How windows hide on workspace switch: "cloak", "hide", "minimize"

[workspaces]
mode = "per-monitor"  # "per-monitor" (default) or "global"
wrap = false          # Stepping past the last workspace goes back to the first

[workspaces.layouts]  # Per-workspace layout overrides
# Available values: "bsp", "vertical-stack", "three-column"
1 = "vertical-stack"
3 = "three-column"

[borders]
width = 4              # Border thickness in pixels (0-32)
corner_style = "small" # "square", "small", or "round"

[borders.colors]
focused = "#00b4d8"    # Color of the focused window in tiled layouts
monocle = "#2d6a4f"    # Color of the focused window in monocle mode
unfocused = "#6c7086"  # Color drawn around unfocused tiled windows
                       # (set to "none" to disable unfocused borders)

[mouse]
follows_focus = true         # Move the pointer to the window that takes focus
focus_follows_mouse = false  # Focus the window under the pointer without a click
drag_to_swap = true          # Drop a dragged tiled window on another to swap them
resize_splits = true         # Drag the edge between two tiles to resize them

[animations]
enabled = true            # Move windows to their new place over time (on by default)
duration = 180            # How long a transition takes, in milliseconds (0-1000)
easing = "ease-out-cubic" # linear, ease-out-quad, ease-out-cubic, ease-in-out-cubic,
                          # ease-out-expo, or "cubic-bezier(x1, y1, x2, y2)"
follow_system = true      # Snap while the system's settings turn animations off,
                          # and in a remote session

[theme]
name = "catppuccin"  # catppuccin, rose-pine, tokyo-night
flavor = "mocha"     # catppuccin: mocha | macchiato | frappe | latte
                     # rose-pine:  main | moon | dawn
                     # tokyo-night: night | storm | day

[logging]
enabled = false    # Enable file logging
level = "info"     # Log level: debug, info, warn, error
max_file_mb = 10   # Max log file size before rotation
```

The `[theme]` section selects the theme, a family and one of its flavors,
whose palette resolves named colors in border and bar configurations. See
[theming.md](theming.md) for details. The `[animations]` section turns on
window transitions; see [animations.md](animations.md) for what moves, what
snaps and why.

The `[mouse]` section ties the pointer to focus. `follows_focus` (default
`true`) moves the pointer to the center of the window that takes focus when
mosaico moves focus itself: on keyboard navigation, a workspace switch, or a
new window it takes in, for example. Moving the focused window takes the
pointer along. Clicking a window, or focusing it with `focus_follows_mouse`,
leaves the pointer where it is. `focus_follows_mouse` (default `false`)
focuses a window once the pointer has rested on it for three polls in a row,
100 ms apart, if the window is managed and on the focused monitor's active
workspace; see [window-events.md](window-events.md#focus-follows-mouse).
While the user drags a window, mosaico moves the pointer for neither, and
focus follows mouse rests: the pointer goes where focus asked once the drag
ends, unless focus has moved on without it by then, as a click or the wheel
on the status bar moves it, and a window the pointer comes to rest on
meanwhile is not focused, though the next one it rests on after the drop
is.

`drag_to_swap` (default `true`) turns the drags the user makes into layout
changes. A tiled window dragged by its title bar and dropped on another tile
of its monitor swaps places with it, and one dropped on another monitor
joins that monitor's workspace before the tile under the pointer, or at the
end, which it also does over that monitor's bar or a gap there. Over its
own slot, a gap, its monitor's bar, a monocle workspace of its own monitor,
or snapped or maximized there by the system as it is dropped, it goes back
to its slot; maximized by the system on another monitor, it joins that
monitor's workspace and stays maximized. A drag cancelled with Escape, and
a click on the title bar, change nothing. While it is dragged, a
translucent preview in the focused border's color shows where it will
land. With `false`, a dropped window goes back to its slot, or joins the end
of the other monitor's workspace, as before. See
[tiling-layout.md](tiling-layout.md#dragging-windows).

`resize_splits` (default `true`) turns resizing a tiled window by its edges
into a layout change. The edge between two tiles moves the split between
them, and the windows beyond it follow; a corner where two splits meet
moves both. In the vertical stack the master's edge and the edges between
the windows of the stack resize, and in three columns the master's edges,
the inner edges of the side columns and the edges between the windows of a
column, with the master staying centered. The edge between two windows of
a stack or a column trades height between those two only, and every other
window keeps its size. The outer edges go back, and a split stops a tenth of
its area from either end, or down a stack where either of the two windows
would have less than a tenth of an equal share of the stack, which can be
under a window's minimum height, so that it overlaps the next, as at the
tenth of a split. Each workspace keeps the splits moved and the heights of
the stacks resized, for each of its layouts: a split nobody moved stays at
`[layout] ratio`, and a stack nobody resized is shared equally, as every
split and stack of the workspaces on screen is again after a retile
(Alt+Shift+R). A window that opens in a resized stack gets an equal share of
it, which the others there give up in proportion. A resize cancelled with
Escape changes nothing, and neither does one of 4 pixels or less on every
edge, as a click. With `false`, a resized window goes back to its slot, as
before. See
[tiling-layout.md](tiling-layout.md#resizing-splits).

### `keybindings.toml`

Maps key combinations to actions (see [keyboard-bindings.md](keyboard-bindings.md)):

```toml
[[keybinding]]
action = "focus-down"
key = "J"
modifiers = ["alt"]
```

### `bar.toml`

Configures the status bar (see [status-bar.md](status-bar.md)):

```toml
enabled = true
height = 28
monitors = []        # 0-based monitor indices; empty for all of them

[mouse]
enabled = true          # false keeps the bar click-through, as before
scroll = "workspaces"   # What scrolling over the bar does: "workspaces" or "none"

# Over the defaults; a later entry for the same widget and button wins
[[mouse.click]]
widget = "clock"        # A widget's type
button = "middle"       # "left", "right" or "middle"
action = "retile"       # An action on a monitor or the whole daemon, or "none"

[colors]
background = "#1e1e2e"   # Hex, or empty for the theme's own
foreground = "#cdd6f4"
accent = "blue"          # A color the theme names, here catppuccin's

[[left]]
type = "workspaces"

[[right]]
type = "clock"
format = "%H:%M"

[[right]]
type = "cpu"
```

The `[mouse]` section says what the mouse does on the bar. A left click on a
workspace number goes to that workspace on the bar's own monitor, and the
wheel steps through that monitor's workspaces the way `goto-workspace-next`
and `goto-workspace-prev` do, with `[workspaces] wrap` from `config.toml`.
`enabled = false` keeps the bar click-through, as it was before it took the
mouse, and `scroll = "none"` leaves the wheel alone. Even with the mouse on,
the bar lets the pointer through over a maximized or fullscreen window, and
elsewhere takes it only over the whole column of the bar each clickable item
stands in, and where it draws something. See
[status-bar.md](status-bar.md#pointer-input).

Clicks on widgets run actions too. By default the layout widget's left and
right buttons cycle the layout and toggle monocle, and a left click on the
paused widget resumes the hotkeys. Each `[[mouse.click]]` entry binds a
button on a kind of widget to an action, named as in `keybindings.toml`, in
place of what that button did there, and a later entry for the same widget
and button wins. `action = "none"` removes the binding, a default included,
and deleting an entry keeps the default. Only an action on the bar's own
monitor or on the whole daemon can be bound: `cycle-layout`,
`toggle-monocle`, `goto-workspace-N`, `goto-workspace-next` and
`goto-workspace-prev` act on the monitor, `retile` and `toggle-pause` on the
whole daemon. An entry with any other action, or with no widget, button or
action, or one that does not exist, is left out, logged and listed by
`mosaico doctor`, and the rest of the file applies. See
[status-bar.md](status-bar.md#widget-clicks).

### `rules.toml`

Controls which windows are managed by the tiler:

```toml
[[rule]]
match_class = "ApplicationFrameWindow"
manage = false
```

## Config Directory

The configuration directory is the config directory of the instance,
`~/.config/mosaico/` for the default instance. The platform crate says where
the user's configuration lives (the `UserDirs` contract in
`mosaico_core::user_dirs`; on Windows `.config` in the profile folder,
`%USERPROFILE%`), and core adds `mosaico` and, for a named instance,
`instances\<name>`. `MOSAICO_CONFIG_DIR` replaces the whole path; see
Instances in [cli.md](cli.md).

`ConfigStore` (in `mosaico_core::config`) reads and writes the config files
in one directory. `ConfigStore::for_instance(instance)` is the store of an
instance, which the daemon, the config watcher and the CLI commands use.
`ConfigStore::new(dir)` points a store at any directory, which is how the
tests load, merge and migrate files in a temporary directory. Its paths:

- `dir()` -> the directory, `~/.config/mosaico/` for the default instance
- `config_path()` -> `config.toml` in it
- `keybindings_path()` -> `keybindings.toml`
- `rules_path()` -> `rules.toml`
- `user_rules_path()` -> `user-rules.toml`
- `bar_path()` -> `bar.toml`

## Loading Behavior

Each config file has two loaders on `ConfigStore`:

- `load_config()` / `load_keybindings()` / `load_rules()` /
  `load_user_rules()` / `load_bar()` -- load with fallback to defaults:
  silently when the file is missing, with a warning when it cannot be read
  or parsed
- `try_load_config()` / `try_load_keybindings()` / `try_load_rules()` /
  `try_load_user_rules()` / `try_load_bar()` -- return
  `mosaico_core::Result<T>`. A failure is an `Error::Config` whose `path` is
  the file and whose message says what is wrong; it reads `<path>: <problem>`

`load_merged_rules()` returns the user rules followed by the community rules.

The `try_load` variants are used by the `doctor` command for validation and
by the config file watcher for hot-reload (only valid configs are applied).

All loaders follow the same pattern:

1. Attempt to read the file from disk
2. For `config.toml`: rewrite keys from older versions (see below)
3. If the file is missing, unreadable, or unparsable: fall back to defaults
   (or return an error for `try_load` variants)
4. For `Config` and `BarConfig`: call `validate()` to clamp values to safe
   ranges. `Config::validate()` also fills the border colors left empty from
   the theme, so `load_config()` validates the defaults it falls back to as
   well. The bar defaults are within every limit already, so the bar falls
   back to them as they are. Its colors are filled from the theme when the
   bar is created, reloaded or rebuilt, and again when the theme changes,
   for a loaded bar and the defaults alike

### Files a Newer Version Updates

A missing file is never created. Files that exist change in two ways:

- Loading `config.toml` rewrites keys from older versions in place
  (`[layout.workspaces]` becomes `[workspaces.layouts]`, and `focused` /
  `monocle` under `[borders]` move to `[borders.colors]`), after copying the
  file to `config.toml.pre-0.9.bak` (`.bak.1`, `.bak.2`, ... when that
  exists). Comments, formatting, the line endings (CRLF or LF) and a byte
  order mark survive. This happens wherever the file is loaded: at daemon
  startup, on hot-reload and in `mosaico doctor`.
- At daemon startup, the `merge_missing_*` methods add what a newer version
  introduced, by appending to the file or, for two keys, inserting into it:
  - `merge_missing_config_sections()` appends a documented `[workspaces]`
    section when `config.toml` has none. A `[workspaces]` table from before
    `wrap` gets `wrap = false` inserted at its end under a comment, the way
    the `unfocused` color is inserted below, and a `workspaces` section in
    another form, such as an inline table, gets a comment block about `wrap`
    instead, unless the file mentions `wrap` anywhere. For the `unfocused`
    border color, when `[borders.colors]` is an active table without the
    key, it inserts `unfocused = ""` there under a comment, even if a
    comment in the file already mentions the option. That rewrites the file
    in place (through `toml_edit`, so comments, formatting, the line endings
    and a byte order mark survive) without a backup. Otherwise it appends a
    comment block about the option, unless the file already mentions
    `unfocused` anywhere. It appends a comment block about `drag_to_swap`
    under `[mouse]`, unless the file mentions it anywhere, as the template
    and the block itself do; the block is all comments and names no default
    value. It appends one about `resize_splits` the same way, and with the
    same rule. Last, it appends a comment block about window transitions,
    unless the file has an `animations` table, as the template writes it,
    or an `[animations]` header line commented out, as the block itself
    has. A line that only mentions the section does not count. The block is
    all comments and names no default value, so the file keeps the defaults
    until the user writes the section, and says nothing wrong about them
    when they change.
  - `merge_missing_keybindings()` and `merge_missing_bar_widgets()` append the
    default bindings and bar widgets the files lack; see
    [keyboard-bindings.md](keyboard-bindings.md) and
    [status-bar.md](status-bar.md). `merge_missing_bar_widgets()` first
    appends a comment block about the bar's `[mouse]` section, unless
    `bar.toml` has a `mouse` table or a `[mouse]` header line commented out,
    the way the animations block is added to `config.toml`. The block tells
    of `enabled`, `scroll` and the `[[mouse.click]]` entries, with the
    actions a click may run and `action = "none"`, and names no default. A
    `bar.toml` that is TOML but holds a value the bar cannot take gets the
    block too, and the bar falls back to the defaults; one that is not TOML
    is left alone.

### Validation

`Config::validate()` clamps values rather than rejecting them:

- `gap`: clamped to `[0, 200]`
- `ratio`: clamped to `[0.1, 0.9]`
- `border width`: clamped to `[0, 32]`
- `[animations] duration`: clamped to `[0, 1000]` milliseconds
- `corner_style`: must be `"square"`, `"small"`, or `"round"`
- `[animations] easing`: must be one of the names above or a
  `cubic-bezier(...)` with four numbers within `[0, 1]`. Like an unknown
  `corner_style`, anything else makes `config.toml` invalid, and the error
  names the file, the line and the easing

In `bar.toml`, an unknown `scroll` makes the file invalid the same way, but a
`[[mouse.click]]` entry is checked on its own as the bar takes the config
(`WidgetClicks`): one with no widget, button or action, one whose widget,
button or action does not exist, or one whose action is about the focused
window, is left out and logged with why, and `mosaico doctor` flags it,
while the rest of the file applies. An entry with `action = "none"` is no
mistake: it removes a binding. A value of the wrong type, such as
`button = 1`, or `[mouse.click]` with single brackets, still makes the file
invalid.

### Partial Files

All config structs use `#[serde(default)]`, so partial TOML files work. A
file containing only `[layout]\ngap = 12` will use defaults for all other
fields.

## Window Rules

Rules determine which windows the tiler should manage. They are evaluated in
order; the first matching rule wins. Windows that match no rule are managed
by default.

### Rule Matching

`matches_rule(class, title, path, rule)`:

- **Class match**: case-insensitive exact match
- **Title match**: case-insensitive substring match
- **Path match**: case-insensitive suffix match against the full path of
  the window's executable, on a path component boundary. `"app.exe"`
  matches `C:\Vendor\app.exe` but not `C:\Vendor\myapp.exe`. `/` and `\`
  are treated the same. The path comes from
  `Window::exe_path()` (`QueryFullProcessImageNameW`) and is empty when
  the process cannot be opened, in which case path rules do not match
- If a rule specifies more than one field, all of them must match
- If a rule specifies none, it matches nothing

### Default Rules

`default_rules()` is empty: the rules come from the community `rules.toml`,
which the daemon downloads, and the user's `user-rules.toml`. The
`rules.toml` that `mosaico init` writes until the first download leaves out
the untitled `ApplicationFrameWindow` frames UWP apps leave behind, and GPG's
`pinentry` prompt.

### Evaluation

`should_manage(class, title, path, rules)`:

1. Iterates rules in order
2. Returns `rule.manage` for the first matching rule
3. If no rule matches, returns `true` (manage the window)

`matching_rule(class, title, path, rules)` returns the same decision as an
`Option<bool>`, with `None` when no rule matches.

### Integration

The tiler's `is_tileable()` (in `crates/mosaico-core/src/tiler/adopt.rs`)
evaluates rules as part of its tileability check, after verifying that the
window is visible and that the platform offers it as a candidate.

`Window::app_window_caption()` rejects tool windows, owned windows, and
modal dialogs outright. For the rest it reports whether the window has
`WS_CAPTION`:

- **With a caption**: `should_manage()` decides, so unmatched windows are
  tiled.
- **Without a caption**: apps that draw their own title bar look the same
  as splash screens and borderless popups, so they are only tiled when
  `matching_rule()` returns `Some(true)`, i.e. a rule explicitly opts
  them in.

## Template Generation

`mosaico init` generates fully-commented template files using functions in
`template.rs`:

- `generate_config()` -- produces `config.toml` with all options documented
- `generate_keybindings()` -- produces `keybindings.toml` with all default
  bindings
- `generate_rules()` -- produces `rules.toml` with default exclusion rules

Templates include comments explaining every option, valid ranges, and
examples. Existing files are not overwritten.

## Hot-Reload

Changes to `config.toml`, `user-rules.toml`, and `bar.toml` are automatically
detected and applied while the daemon is running. The config file watcher
(see [daemon.md](daemon.md)) waits for the platform to report a change in
the config directory, then reloads each of the three files whose
modification time changed. `ConfigReloader` and `ConfigWatch`
(`mosaico_core::config`) make these decisions, so every platform reloads the
same way.

- **config.toml**: gap/ratio, hiding behaviour, border settings, the
  `[mouse]` settings, `[animations]` and theme are reloaded. The tiling
  manager calls `reload_config()` which updates the layout gap/ratio, hiding
  strategy, `BorderConfig`, whether the mouse follows focus, whether a drop
  moves a dragged window (`drag_to_swap`, from its next report or drop),
  whether a resize moves splits (`resize_splits`, read as a resize ends, so
  from the next one that ends, one held through the reload included) and the
  `[animations]` settings, then retiles all windows, with a new ratio for
  every split the user did not move; the daemon starts or
  stops focus-follows-mouse. Transitions that start after the reload use the
  new settings. A motion in flight goes on as it is while its target stays the
  same, a reload that moves its target turns it there with the new duration,
  and one that turns transitions off ends it with a placement that snaps.
  Bar colors, the `[colors]` table and each widget's `color`, are resolved
  again against the theme, so they follow a theme change. Hiding changes
  take effect on the next workspace switch; a strategy the platform does not
  have is swapped for one it has, with a warning in the log, as at startup.
  Windows hidden before the change are shown again the way they were hidden.
  The workspace mode
  (`workspaces.mode`) and `workspaces.wrap` are reloaded live. Turning `"global"` on switches every
  monitor to the focused monitor's workspace, hiding and showing windows as
  a workspace switch does; turning it off leaves each monitor on the
  workspace it shows. Per-workspace layout overrides
  (`[workspaces.layouts]` and `layout.default`) are applied at startup only;
  reloading does not override the active layout chosen via `cycle-layout`
  or `set-layout`; restart the daemon to pick up changes.
- **user-rules.toml**: the window rules (user rules first, then the community
  rules from `rules.toml`) are replaced via `reload_rules()`. New rules apply
  to new windows. Managed windows the new rules no longer manage are released
  at once, which the log shows as `-rule <window> (unmanaged by new rules)`,
  and their monitor is retiled. A window hidden on another workspace is out
  of view only because the tiler hid it, so it is judged by the rules alone,
  on what `Desktop::facts` says about it, and a hidden window the rules
  release is shown again the way it was hidden. A window the new rules would
  manage is picked up the next time it gets focus.
- **rules.toml**: **not** watched. The daemon downloads the community rules
  at startup and reloads them itself once they are saved, and the next
  download overwrites hand edits. Put your own rules in `user-rules.toml`.
- **bar.toml**: the `BarManager` is recreated with `reload()`, colors are
  re-resolved against the current theme, work areas are reset and re-adjusted
  for the new bar height, and all monitors are retiled. The bars are made
  again, with the new `[mouse]` settings, so a press or wheel turns made on
  the old ones are dropped, and each `[[mouse.click]]` entry the new file
  has that a bar cannot run is logged.
- **keybindings.toml**: **not** hot-reloaded. Hotkey changes require a daemon
  restart because `RegisterHotKey` binds at the Win32 thread level.

Only valid configurations are applied. If a change to `config.toml` or
`bar.toml` introduces a parse error, the watcher logs it
(`config.toml invalid, skipping: ...`) and keeps the current config until the
file changes again. `user-rules.toml` is different: a file that does not
parse counts as no user rules, as it does at startup, so the community rules
apply alone until it is fixed.

## Design Decisions

- **Defaults everywhere**: every config field has a `#[serde(default)]` so
  the application works without any configuration files.
- **Clamp, don't reject**: validation clamps out-of-range values to safe
  boundaries rather than refusing to start, prioritizing usability.
- **First-match-wins rules**: simple and predictable evaluation order. The
  default behavior (manage everything) is sensible for most users.
- **Case-insensitive matching**: prevents frustration with class/title casing
  inconsistencies across Windows applications.
- **Separate files**: splitting config, keybindings, rules, and bar into
  separate files makes each concern independently manageable.
- **try_load variants**: provide structured error reporting for the `doctor`
  command and config watcher, while the plain `load` variants silently fall
  back to defaults for daemon startup resilience.

## Tests

- 29 unit tests in `config/tests.rs` covering: defaults, partial TOML
  parsing, rule matching (by class, by title, by path, first-match wins,
  no-rules default), validation clamping
- 8 tests of `[animations]` in `config/animations_tests.rs`: the defaults,
  parsing, the duration clamped, `spec()` and `is_on()`, and an unknown
  easing or a `cubic-bezier` that would overshoot making `config.toml`
  invalid; the easing names in `motion/tests/easing_names_tests.rs`
- Template tests (`template_tests.rs`, `template_animations_tests.rs`):
  roundtrip parsing, template-matches-defaults verification, and that the
  `[animations]` section writes out every key and names only easings the
  config takes
- `ConfigStore` tests in `config/store/tests/`. The tests of the migration
  and merge helpers work on single files in the system temp directory. The
  tests of the store's operations each get a directory of their own, deleted
  when the test ends: every load and its fallback, the errors that name the
  file, the migrations of `config.toml` with their backups, and each merge of
  new defaults, including what it adds, that a second run changes nothing, and
  the daemon's order of merging before loading
- Hot-reload tests (`config/reload/tests/`): which file changes reload what
  and in which order, invalid and deleted files, that only a new modification
  time counts as a change, and the watch loop against a scripted
  `DirectoryWatcher` (a change made while the watcher opens is not lost, a
  stop wins over a change, a quiet wait loads nothing, each wait lasts at
  most 5 seconds, a failed wait ends the watch)

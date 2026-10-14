# Configuration System & Window Rules

Mosaico uses four TOML configuration files stored in `~/.config/mosaico/`.
All settings have sensible defaults, so configuration files are optional.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/config/mod.rs` | `Config`, `LayoutConfig`, `BorderConfig`, `ThemeConfig`, `WindowRule`, `should_manage()` |
| `crates/mosaico-core/src/config/hiding_behaviour.rs` | `HidingBehaviour`, how windows are hidden on a workspace switch |
| `crates/mosaico-core/src/config/store/` | `ConfigStore`: the paths of the config files (`mod.rs`), loading them (`load.rs`), each migration (`migrate*.rs`) and each merge of new defaults (`merge_*.rs`) |
| `crates/mosaico-core/src/config/reload/` | `ConfigReloader`, `ConfigReload`: which files hot-reload and how (`mod.rs`); `ConfigWatch`: opening the watcher after the files are recorded, and the watch loop (`run.rs`) |
| `crates/mosaico-core/src/config/keybinding.rs` | `Keybinding`, `Modifier`, `defaults()`, `Keybinding::combo()` |
| `crates/mosaico-core/src/config/bar.rs` | `BarConfig`, `BarColors`, `WidgetConfig` |
| `crates/mosaico-core/src/config/theme.rs` | `ThemeConfig`, `Theme` enum |
| `crates/mosaico-core/src/config/palette.rs` | Catppuccin color palettes (hardcoded named color data) |
| `crates/mosaico-core/src/config/template.rs` | Template generators for `mosaico init` |

### Key Types

- `Config` -- top-level config: `layout: LayoutConfig`, `borders: BorderConfig`,
  `logging: LogConfig`, `theme: ThemeConfig`
- `LayoutConfig` -- `default: String` (default `"bsp"`), `gap: i32` (default 8),
  `ratio: f64` (default 0.5), `hiding: HidingBehaviour` (default `Cloak`),
  `workspaces: HashMap<u32, String>` (per-workspace layout overrides)
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
- `LogConfig` -- `enabled: bool` (default false), `level: String` (default
  `"info"`), `max_file_mb: u64` (default 10)
- `WindowRule` -- `match_class: Option<String>`, `match_title: Option<String>`,
  `match_path: Option<String>`, `manage: bool`
- `ThemeConfig` -- `flavor: Option<String>` (default `None`, resolves to
  `Theme::Mocha`)
- `Theme` (enum) -- `Latte`, `Frappe`, `Macchiato`, `Mocha`
- `BarConfig` -- `enabled: bool`, `height: i32`, `monitor: String`,
  `left/center/right: Vec<WidgetConfig>`, `colors: BarColors`
- `BarColors` -- `background`, `foreground`, `accent` (support named colors)
- `KeybindingsFile` -- wrapper for TOML deserialization of `[[keybinding]]`
  arrays
- `RulesFile` -- wrapper for TOML deserialization of `[[rule]]` arrays

## Configuration Files

### `config.toml`

Controls layout, borders, logging, and theme:

```toml
[layout]
default = "bsp"  # Default layout algorithm: "bsp", "vertical-stack", "three-column"
gap = 8          # Pixel gap between windows (0-200)
ratio = 0.5      # BSP split ratio (0.1-0.9)
hiding = "cloak" # How windows hide on workspace switch: "cloak", "hide", "minimize"

[workspaces]
mode = "per-monitor"  # "per-monitor" (default) or "global"

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

The `[theme]` section selects the Catppuccin color palette used for resolving
named colors in border and bar configurations. See
[theming.md](theming.md) for details.

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
monitor = "all"      # "all", "primary", or 0-based index

[colors]
background = "base"      # Named Catppuccin color
foreground = "text"
accent = "blue"

[[left]]
type = "workspaces"

[[center]]
type = "clock"
format = "%H:%M"

[[right]]
type = "cpu"
```

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
  exists). Comments and formatting survive. This happens wherever the file is
  loaded: at daemon startup, on hot-reload and in `mosaico doctor`.
- At daemon startup, the `merge_missing_*` methods add what a newer version
  introduced, by appending to the file or, for one key, inserting into it:
  - `merge_missing_config_sections()` appends a documented `[workspaces]`
    section when `config.toml` has none. For the `unfocused` border color,
    when `[borders.colors]` is an active table without the key, it inserts
    `unfocused = ""` there under a comment, even if a comment in the file
    already mentions the option. That rewrites the file in place (through
    `toml_edit`, so comments and formatting survive) without a backup.
    Otherwise it appends a comment block about the option, unless the file
    already mentions `unfocused` anywhere.
  - `merge_missing_keybindings()` and `merge_missing_bar_widgets()` append the
    default bindings and bar widgets the files lack; see
    [keyboard-bindings.md](keyboard-bindings.md) and
    [status-bar.md](status-bar.md).

### Validation

`Config::validate()` clamps values rather than rejecting them:

- `gap`: clamped to `[0, 200]`
- `ratio`: clamped to `[0.1, 0.9]`
- `border width`: clamped to `[0, 32]`
- `corner_style`: must be `"square"`, `"small"`, or `"round"`

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

`default_rules()` excludes:

- `ApplicationFrameWindow` class -- UWP apps like Settings and Microsoft Store

### Evaluation

`should_manage(class, title, path, rules)`:

1. Iterates rules in order
2. Returns `rule.manage` for the first matching rule
3. If no rule matches, returns `true` (manage the window)

`matching_rule(class, title, path, rules)` returns the same decision as an
`Option<bool>`, with `None` when no rule matches.

### Integration

`TilingManager::is_tileable()` evaluates rules as part of its tileability
check, after verifying visibility and window style.

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

- **config.toml**: gap/ratio, hiding behaviour, border settings, and theme
  are reloaded. The tiling manager calls `reload_config()` which updates the
  layout gap/ratio, hiding strategy, and `BorderConfig`, then retiles all
  windows. Bar colors, the `[colors]` table and each widget's `color`, are
  resolved again against the theme, so they follow a theme change. Hiding changes
  take effect on the next workspace switch; a strategy the platform does not
  have is swapped for one it has, with a warning in the log, as at startup.
  Windows hidden before the change are shown again the way they were hidden.
  The workspace mode
  (`workspaces.mode`) is reloaded live. Per-workspace layout overrides
  (`[workspaces.layouts]` and `layout.default`) are applied at startup only;
  reloading does not override the active layout chosen via `cycle-layout`
  or `set-layout`; restart the daemon to pick up changes.
- **user-rules.toml**: the window rules (user rules first, then the community
  rules from `rules.toml`) are replaced via `reload_rules()`. New rules apply
  to new windows. Managed windows the new rules no longer manage are released
  at once, which the log shows as `-rule <window> (unmanaged by new rules)`,
  and their monitor is retiled. A window the new rules would manage is
  picked up the next time it gets focus.
- **rules.toml**: **not** watched. The daemon downloads the community rules
  at startup and reloads them itself once they are saved, and the next
  download overwrites hand edits. Put your own rules in `user-rules.toml`.
- **bar.toml**: the `BarManager` is recreated with `reload()`, colors are
  re-resolved against the current theme, work areas are reset and re-adjusted
  for the new bar height, and all monitors are retiled.
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

- 11 unit tests covering: defaults, partial TOML parsing, rule matching
  (by class, by title, first-match wins, no-rules default), validation
  clamping
- 6 template tests: roundtrip parsing, template-matches-defaults verification
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

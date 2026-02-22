# Configuration System & Window Rules

Mosaico uses three TOML configuration files stored in `~/.config/mosaico/`.
All settings have sensible defaults, so configuration files are optional.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/config/mod.rs` | `Config`, `LayoutConfig`, `BorderConfig`, `WindowRule`, `should_manage()` |
| `crates/mosaico-core/src/config/loader.rs` | `load()`, `try_load()`, `load_keybindings()`, `try_load_keybindings()`, `load_rules()`, `try_load_rules()`, path helpers |
| `crates/mosaico-core/src/config/keybinding.rs` | `Keybinding`, `Modifier`, `defaults()` |
| `crates/mosaico-core/src/config/template.rs` | Template generators for `mosaico init` |

### Key Types

- `Config` -- top-level config: `layout: LayoutConfig`, `borders: BorderConfig`,
  `logging: LogConfig`
- `LayoutConfig` -- `gap: i32` (default 8), `ratio: f64` (default 0.5)
- `BorderConfig` -- `width: i32` (default 4), `focused: String` (default
  `"#00b4d8"`), `monocle: String` (default `"#2d6a4f"`)
- `LogConfig` -- `enabled: bool` (default false), `level: String` (default
  `"info"`), `max_file_mb: u64` (default 10)
- `WindowRule` -- `match_class: Option<String>`, `match_title: Option<String>`,
  `manage: bool`
- `KeybindingsFile` -- wrapper for TOML deserialization of `[[keybinding]]`
  arrays
- `RulesFile` -- wrapper for TOML deserialization of `[[rule]]` arrays

## Configuration Files

### `config.toml`

Controls layout, borders, and logging:

```toml
[layout]
gap = 8       # Pixel gap between windows (0-200)
ratio = 0.5   # BSP split ratio (0.1-0.9)

[borders]
width = 4              # Border thickness in pixels (0-32)
focused = "#00b4d8"    # Hex color for focused window
monocle = "#2d6a4f"    # Hex color for monocle mode

[logging]
enabled = false    # Enable file logging
level = "info"     # Log level: debug, info, warn, error
max_file_mb = 10   # Max log file size before rotation
```

### `keybindings.toml`

Maps key combinations to actions (see [keyboard-bindings.md](keyboard-bindings.md)):

```toml
[[keybinding]]
action = "focus-down"
key = "J"
modifiers = ["alt"]
```

### `rules.toml`

Controls which windows are managed by the tiler:

```toml
[[rule]]
match_class = "ApplicationFrameWindow"
manage = false
```

## Config Directory

The configuration directory is `~/.config/mosaico/`, resolved via
`dirs::home_dir()`. The `config_dir()` function returns this path.

Individual file paths:
- `config_path()` -> `~/.config/mosaico/config.toml`
- `keybindings_path()` -> `~/.config/mosaico/keybindings.toml`
- `rules_path()` -> `~/.config/mosaico/rules.toml`

## Loading Behavior

Each config file has two loader variants:

- `load()` / `load_keybindings()` / `load_rules()` -- load with silent
  fallback to defaults on any error
- `try_load()` / `try_load_keybindings()` / `try_load_rules()` -- return
  `Result<T, String>` with a descriptive error message on failure

The `try_load` variants are used by the `doctor` command for validation and
by the config file watcher for hot-reload (only valid configs are applied).

All loaders follow the same pattern:

1. Attempt to read the file from disk
2. If the file is missing, unreadable, or unparsable: fall back to defaults
   (or return an error for `try_load` variants)
3. For `Config`: call `validate()` to clamp values to safe ranges

### Validation

`Config::validate()` clamps values rather than rejecting them:

- `gap`: clamped to `[0, 200]`
- `ratio`: clamped to `[0.1, 0.9]`
- `border width`: clamped to `[0, 32]`

### Partial Files

All config structs use `#[serde(default)]`, so partial TOML files work. A
file containing only `[layout]\ngap = 12` will use defaults for all other
fields.

## Window Rules

Rules determine which windows the tiler should manage. They are evaluated in
order; the first matching rule wins. Windows that match no rule are managed
by default.

### Rule Matching

`matches_rule(class, title, rule)`:

- **Class match**: case-insensitive exact match
- **Title match**: case-insensitive substring match
- If a rule specifies both `match_class` and `match_title`, both must match
- If a rule specifies neither, it matches everything

### Default Rules

`default_rules()` excludes:

- `ApplicationFrameWindow` class -- UWP apps like Settings and Microsoft Store

### Evaluation

`should_manage(class, title, rules)`:

1. Iterates rules in order
2. Returns `rule.manage` for the first matching rule
3. If no rule matches, returns `true` (manage the window)

### Integration

`TilingManager::is_tileable()` calls `should_manage()` as part of its
tileability check, after verifying visibility and window style.

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

Changes to `config.toml` and `rules.toml` are automatically detected and
applied while the daemon is running. The config file watcher (see
[daemon.md](daemon.md)) polls for modification time changes every 2 seconds.

- **config.toml**: layout gap/ratio and border settings are reloaded. The
  tiling manager calls `reload_config()` which updates the `BspLayout` and
  `BorderConfig`, then retiles all windows.
- **rules.toml**: window rules are replaced via `reload_rules()`. New rules
  apply to newly created windows; existing managed windows are not re-evaluated.
- **keybindings.toml**: **not** hot-reloaded. Hotkey changes require a daemon
  restart because `RegisterHotKey` binds at the Win32 thread level.

Only valid configurations are applied. If a file change introduces a parse
error, the watcher logs a warning and keeps the current config.

## Design Decisions

- **Defaults everywhere**: every config field has a `#[serde(default)]` so
  the application works without any configuration files.
- **Clamp, don't reject**: validation clamps out-of-range values to safe
  boundaries rather than refusing to start, prioritizing usability.
- **First-match-wins rules**: simple and predictable evaluation order. The
  default behavior (manage everything) is sensible for most users.
- **Case-insensitive matching**: prevents frustration with class/title casing
  inconsistencies across Windows applications.
- **Separate files**: splitting config, keybindings, and rules into separate
  files makes each concern independently manageable.
- **try_load variants**: provide structured error reporting for the `doctor`
  command and config watcher, while the plain `load` variants silently fall
  back to defaults for daemon startup resilience.

## Tests

- 11 unit tests covering: defaults, partial TOML parsing, rule matching
  (by class, by title, first-match wins, no-rules default), validation
  clamping
- 6 template tests: roundtrip parsing, template-matches-defaults verification

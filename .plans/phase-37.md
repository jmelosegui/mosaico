# Phase 37: Tray Icon & Settings App

**Status:** Pending

**Goal:** Let people configure mosaico without editing TOML. A tray icon
gives the daemon a visible home. From the icon, a settings window offers only
valid values for each setting, saves them to the same config files, and lets
hot-reload apply them.

## Overview

Today every setting lives in five TOML files in the config directory
(`config.toml`, `bar.toml`, `keybindings.toml`, `rules.toml`,
`user-rules.toml`). People find them with `mosaico init` and the docs, and
learn the valid values from the comments, the website or `mosaico doctor`.
A typo is clamped or ignored on reload, and the only feedback is the log.

This phase adds two things:

1. **A tray icon**, owned by the daemon while it runs. Its menu opens the
   settings, pauses and resumes tiling, reloads the config, opens the config
   folder and the log, and quits. The icon shows when tiling is paused.
2. **A settings app**, `mosaico-settings`, a separate executable. It is
   built on egui/eframe, the UI stack of the ArtCraft apps
   ([storytold/craft-libs](https://github.com/storytold/craft-libs)), with
   their look and feel: design tokens, the Inter font, spacing and widgets.
   Its colors come from the active mosaico theme. One window, one code base,
   on every OS mosaico supports.

The settings app never decides what is valid. Core describes every setting:
its type, range, choices, default and help text. Validation and clamping use
the same description, so the app, the config loader and `mosaico doctor`
cannot disagree. Saving edits the files with `toml_edit`, so comments, order
and formatting the user wrote survive. The daemon's existing hot-reload
(phase 13) applies the change. The app needs no new IPC to change settings.

## Design Goals

- **Only valid values.** Enums are drop-downs, ranges are sliders or spin
  boxes clamped to their bounds, and colors are pickers that also accept
  theme palette names. A keybinding editor captures keys and refuses
  combinations core cannot parse. Nothing the app can save makes the loader
  clamp, ignore or warn.
- **One source of truth.** The setting descriptions in core feed the UI, the
  loader's validation, the templates `mosaico init` writes, and `mosaico
  doctor`. A test fails if a field exists in a config struct without a
  description, or a description without a field.
- **The files stay the user's.** Comments, key order, blank lines, line
  endings and the byte order mark survive a save (`write_back` already does
  the last two). A setting left at its default is not written unless the
  file already has it. Hand edits and the app can be used side by side.
- **No surprises from concurrent edits.** If a file changes on disk while
  the app has unsaved edits to it, the app says so and offers reload or
  overwrite. It never silently overwrites.
- **Cross-platform by construction.** `mosaico-settings` depends on
  `mosaico-core` and egui only. Anything that touches the OS (single
  instance, opening a folder, autostart) goes through core traits, picked
  in a small composition root as in the CLI's `platform.rs`. The crate builds
  for Linux and macOS targets in CI even before those platforms ship.
- **mosaico.exe stays small.** egui, eframe and the renderer never link into
  `mosaico.exe`. The tray uses the platform's native API from the platform
  crate, and the 2,000,000-byte budget (`cargo xtask size`) still holds.
- **The daemon never waits on the UI.** The tray runs on the daemon's
  message loop and only posts commands, as hotkeys do. The settings app is
  a separate process, so a slow or crashed window cannot stall tiling.
- **Testable in isolation.** Setting descriptions, the document editor and
  the tray menu model are pure core code with unit tests. The app's pages are
  tested headless with `egui_kittest`, as the ArtCraft apps are, against a
  config store on a temporary directory.

## Configuration

```toml
# config.toml
[tray]
# Show the mosaico icon in the notification area while the daemon runs.
enabled = true
```

The settings app keeps its own preferences, such as window size and
position, in `settings-app.toml` in the config
directory. The daemon never reads that file and hot-reload ignores it.

## The Tray Icon

| Item | Action |
| --- | --- |
| **Settings…** (default, also on left click) | Starts `mosaico-settings`, or brings its window forward if one is open |
| Pause tiling / Resume tiling | The `toggle-pause` action (phase 29); the label follows the state |
| Reload config | Same as a config file change: reload and report problems |
| Open config folder | Opens the config directory in the file manager |
| Open log | Opens `mosaico.log` |
| Quit mosaico | The `stop` command |

- The tooltip shows `mosaico <version>`, plus `(paused)` while paused.
- The icon has a running and a paused variant, and light and dark variants
  for the taskbar theme. The paused variant is dimmed with a red pause badge,
  matching the bar's paused widget, which is red by default.
- If the shell restarts (Explorer crashes), the icon comes back.
- `[tray] enabled = false` removes the icon on reload, and `true` adds it.
- A config reload that fails shows a balloon notification. Its text comes
  from the same report as the log line.

## The Settings App

One window with a navigation list on the left and a page on the right:

| Page | File | Contents |
| --- | --- | --- |
| General | config.toml | Theme and flavor (Catppuccin, Rosé Pine, Tokyo Night), tray, logging, autostart |
| Layout | config.toml | Default layout, gap, ratio, hiding behaviour |
| Workspaces | config.toml | Mode, wrap, per-workspace layouts |
| Borders | config.toml | Width, corner style, focused/monocle/unfocused colors |
| Mouse | config.toml | Drag to swap, split resizing, focus follows mouse, mouse follows focus |
| Animations | config.toml | Enabled, duration, easing, follow the system setting |
| Bar | bar.toml | Enabled, monitors, height, font, padding and pills, opacity, colors, left/right widgets and their order, bar mouse actions |
| Keybindings | keybindings.toml | Table of action ↔ keys, key capture, conflict warnings |
| Rules | user-rules.toml | Rules with match fields; `rules.toml` (community rules) read-only |

- **Apply** saves every changed file. **Revert** discards unsaved edits.
  Closing with unsaved edits asks first.
- Each field shows its default, and a reset button appears when the value
  differs from the default.
- Help text under each field comes from the same descriptions as the
  template comments.
- Labels describe what the user sees, never how an OS does it: "Keep in
  taskbar", not "Cloak". Choices that depend on the platform come from the
  platform's capabilities (as hiding does through `HidingSupport`), and
  choices it lacks are not shown. There are no OS-specific pages. The value
  written to the file stays the config name (`cloak`).
- A status line shows whether the daemon is running and paused (the existing
  `status` IPC request). After Apply, it shows whether the daemon accepted
  the reload. The app works the same with no daemon running; the next start
  reads the files.
- Only one settings window runs per instance. A second launch brings the
  first forward.

### Look and feel

Colors come from mosaico's theme. Everything else follows the ArtCraft apps
rather than egui's defaults:

- Colors and backgrounds come from the active theme flavor (for example
  Catppuccin Mocha or Latte), so the app matches the borders and the bar.
  Light or dark follows the flavor, and the app updates live when the theme
  changes in `config.toml`. There is no separate Light/Dark/System choice.
- Core gives each flavor the UI roles the app needs beyond `BarColors`:
  darker panels (title bar, sidebar), dim text, hover and pressed. Dark
  flavors put cards above the page. Light flavors make cards the lightest
  surface, as ArtCraft's light theme does.
- Tokens are applied to egui `Visuals` as in `deckcraft-ui-egui::theme`, and
  widget states use `craft_ui::theme::WidgetPalette` from craft-libs, so the
  shared styling stays shared.
- The Inter UI font, 13 pt body and buttons, 11 pt small text, 18 pt
  headings, and the same spacing and corner radii.
- A title bar, a sidebar with search and page navigation, and settings
  grouped in bordered cards. Sketches are in
  [`design/settings`](../design/settings).

## Architecture

### Core

- `config/schema/`: `SettingDescriptor { key, file, section, label, help,
  kind, default }` with `SettingKind::{Bool, Int { min, max }, Float { min,
  max, step }, Choice(&[..]), Color, Font, Keys, Text, List(..)}`. Constants hold
  the ranges, and `Config::validate`, `BarConfig` validation and the
  descriptors all read those constants.
- `config/document/`: `SettingsDocument` wraps a `toml_edit::DocumentMut`
  per file. It provides `get`, `set` (typed and checked against the
  descriptor), `reset`, `is_dirty`, `has_changed_on_disk` and `save` through
  `ConfigStore` and `write_back`. Saving parses the result with the normal
  loader first and refuses anything the loader would clamp or reject.
- `tray/`: `TrayMenu` and `TrayItem` model the menu and its labels from the
  daemon state. `TrayCommand` is what a click produces. The daemon maps a
  `TrayCommand` to the actions and commands it already has.
- Platform contracts (doc comments name no OS concepts):
  - `TrayIcon`: `show(&TrayState)`, `update(&TrayState)`, `notify(title,
    body)`, `hide()`. Clicks arrive as `TrayCommand` events on the daemon
    loop.
  - `Launcher`: `open_path(&Path)` and `spawn_detached(&Path, args)` for
    starting the settings app and opening folders and the log.
  - `FontCatalog`: the installed font families, so the bar font is chosen
    from a list rather than typed.
  - The settings app reuses the single-instance guard from phase 24
    through its existing core trait, with its own name.

### mosaico-windows

- `tray/`: `Shell_NotifyIconW` on a message-only window owned by the
  daemon thread. It re-adds the icon on `TaskbarCreated`, builds the menu
  with `TrackPopupMenu` from a `TrayMenu`, and calls
  `NIM_SETVERSION` (`NOTIFYICON_VERSION_4`). Icons are embedded resources.
- `Launcher` uses `ShellExecuteW` and `CreateProcessW` with
  `DETACHED_PROCESS`.
- `FontCatalog` uses DirectWrite's system font collection.

### mosaico-settings (new crate)

- `[[bin]] mosaico-settings`, with a `windows_subsystem = "windows"`
  attribute in `main.rs` only.
- Dependencies: `mosaico-core`, `eframe`/`egui` (0.36, as ArtCraft),
  `craft-ui` (git rev pinned), and dev-dependency `egui_kittest`.
- `app/` (state, Apply/Revert), `theme/` (tokens from the active flavor, fonts),
  `pages/` (one module per page), `widgets/` (setting row, color picker, key
  capture, list editor).
- `platform.rs` is the composition root that picks the platform crate per
  target, as the CLI does.

### mosaico (CLI)

- `mosaico settings` starts the app, the same way the tray does.
- `mosaico doctor` reports with the descriptors, so its messages match the
  app's.

## Edge Cases

- The config directory or a file does not exist. The app shows defaults, and
  Apply creates only the files that have changes.
- A file does not parse. Its pages show the parse error and stay read-only
  until the file is fixed or reset. The other files stay editable.
- A file is changed by hand while the app is open (see Design Goals).
- A key the descriptors don't know, from a newer or older mosaico, or a typo.
  It is preserved on save and listed as unknown on the page.
- Values written as theme palette names (`"blue"`) versus hex. The picker
  keeps whichever form the file used unless the user picks the other.
- The daemon is not running, is paused, or runs another version. The status
  line says so. Saving still works.
- Several instances (`Instance`). The app edits the instance it was started
  for, and the title bar names it when it isn't the default one.
- DPI changes and monitors with different scaling, in eframe.
- The tray is hidden in the overflow area. Nothing depends on it being
  visible; `mosaico settings` is always available.

## Later

- Live preview: apply gap, border and theme changes to the running daemon
  before Apply, through a preview IPC request.
- A visual layout and monitor map.
- Importing and exporting a settings profile.
- Localization, when the ArtCraft apps share it.

## New and Modified Files

- `crates/mosaico-core/src/config/schema/*`, `config/document/*`, `tray/*`
- `crates/mosaico-core/src/config/mod.rs`, `config/types.rs` (`TrayConfig`,
  ranges as constants), templates and hints
- `crates/mosaico-windows/src/tray/*`, `launcher.rs`, icon resources
- `crates/mosaico-core/src/config/palette/*` (per-flavor UI roles)
- `crates/mosaico-settings/` (new)
- `crates/mosaico/src/commands/settings.rs`, `doctor*.rs`, the daemon's
  event loop
- `xtask`: `size` and `metrics` learn about the second executable
- Release workflow, `mosaico update`, the installer (phase 30)
- `docs/settings-app.md`, `docs/configuration.md`, website guide
- `design/settings/` (sketches)

## Tasks

Every step runs fmt, clippy, the safe test suites and `cargo xtask metrics`,
and brings its docs, so it can merge on its own.

### 37.1 Setting descriptions

- [ ] `SettingDescriptor` and `SettingKind` in core, with descriptors for
      every field of `config.toml` and `bar.toml`
- [ ] Ranges as constants shared by validation and descriptors
- [ ] Tests: every config field has a descriptor and the reverse; every
      default validates unchanged; every out-of-range value is clamped to the
      descriptor's bounds
- [ ] `mosaico doctor` messages come from the descriptors

### 37.2 Editing config files

- [ ] `SettingsDocument`: get, set, reset, dirty tracking, change-on-disk
      detection, save through `ConfigStore`
- [ ] Tests on a temporary store: comments, order, CRLF and BOM survive;
      defaults aren't added; unknown keys are kept; invalid sets are refused;
      a file changed on disk is detected

### 37.3 Tray icon

- [ ] `TrayIcon` and `Launcher` contracts and the `TrayMenu` model in core,
      with tests for labels per state and the command each item produces
- [ ] Windows implementation: icon, menu, tooltip, paused variant, taskbar
      restart, balloon on a failed reload
- [ ] Daemon wiring and `[tray] enabled` with hot-reload; template and
      hints
- [ ] Manual: every menu item, Explorer restart, toggling `enabled`

### 37.4 Settings app shell

- [ ] Per-flavor UI roles in core's theme palettes
- [ ] `mosaico-settings` crate, ArtCraft tokens with colors from the
      active flavor, Inter font, live theme changes, `settings-app.toml`
- [ ] Navigation, setting rows from descriptors, Apply/Revert, unsaved-edit
      prompt, daemon status line, single instance
- [ ] `mosaico settings` and the tray's Settings… item start it
- [ ] `egui_kittest` tests for the shell in a dark and a light flavor
- [ ] CI builds the crate for Linux and macOS targets

### 37.5 Pages for config.toml and bar.toml

- [ ] General, Layout, Workspaces, Borders, Mouse, Animations, Bar
- [ ] Color picker with palette names, widget order editor
- [ ] `egui_kittest` tests: each page edits, saves and reloads through a
      temporary store

### 37.6 Keybindings and rules

- [ ] Key capture validated by core's keybinding parser, conflict
      detection in core
- [ ] Rules list editor for `user-rules.toml`; community rules read-only
- [ ] Tests for capture, conflicts and round-trips

### 37.7 Shipping

- [ ] Release artifacts and `mosaico update` include `mosaico-settings`
- [ ] `cargo xtask size` keeps checking `mosaico.exe` and reports the
      settings app's size separately
- [ ] Docs, website guide with screenshots in a dark and a light flavor,
      cheatsheet

### Verification

- [ ] `mosaico.exe` is under 2,000,000 bytes, and no egui crate appears in
      its dependency tree
- [ ] A setting changed in the app is applied by the running daemon without
      a restart
- [ ] A hand-commented config file keeps its comments after Apply
- [ ] The app follows a theme or flavor change in `config.toml` live

## Open Decisions

| Question | Proposal |
| --- | --- |
| Separate executable or a feature flag on `mosaico.exe` | Separate `mosaico-settings` executable: keeps the size budget and keeps a UI crash away from the daemon |
| Depend on `craft-ui` by git or copy the tokens | Depend on `craft-ui` (pinned rev) for shared widget styling and copy the per-app tokens, as each ArtCraft app does |
| Renderer | The renderer the ArtCraft apps ship with (`glow` or `wgpu`), whichever keeps the executable smaller |
| Where app preferences live | `settings-app.toml` in the config directory, or `craft-settings` storage for consistency with ArtCraft |
| Scope of the first release | 37.1–37.5 first; keybindings and rules (37.6) can follow |

# Phase 38: Launch Applications from Keybindings

**Status:** In Progress

**Goal:** Let a key combination start an application, such as Alt+Shift+B for
the browser or Alt+Enter for a terminal, and, when the user asks for it, bring
an open window of that application into view instead of starting another one.
Requested in #207.

## Overview

Every keybinding today runs a tiler [`Action`]: a fixed set of names such as
`focus-left` or `goto-workspace-3`. Starting a program is a different kind of
command. It carries data the user writes (a command and its arguments), it
talks to the OS rather than the tiler, and it may first look for a window the
tiler manages.

This phase adds a second kind of binding, a launch binding, in the same
`keybindings.toml`. The hotkey layer registers both kinds and tells the daemon
which one fired. The daemon either asks the tiler to focus a matching window or
asks a new platform contract to start the program.

## Design Goals

- **No change to `Action`.** It stays a `Copy` name that the CLI, IPC, bar
  clicks and the settings app already share. Launch bindings live beside it.
- **Old versions keep working.** A `keybindings.toml` with launch bindings must
  still load in an older daemon or settings app, which ignore what they do not
  know instead of rejecting the whole file and falling back to defaults.
- **Portable.** The decision (parse, validate, match, focus or launch) is in
  `mosaico-core`. The platform only starts a process, through a new contract
  that a Linux or macOS crate can implement.
- **Nothing on by default.** No launch bindings ship. Which browser or terminal
  someone uses is their choice; the template and docs show examples.
- **Testable.** The launcher is a trait, so the daemon and the tiler are tested
  against a fake that records what was started.

## Configuration

```toml
# keybindings.toml

# Start a new terminal every time.
[[launch]]
command = "wt.exe"
key = "Enter"
modifiers = ["alt"]

# Bring an open Edge window into view, or start Edge when none is open.
[[launch]]
command = "msedge.exe"
args = ["--new-window"]
key = "B"
modifiers = ["alt", "shift"]
focus = true

# Windows Terminal's windows belong to WindowsTerminal.exe, not wt.exe, so say
# which windows count as already open.
[[launch]]
command = "wt.exe"
key = "T"
modifiers = ["alt", "shift"]
focus = true
match_path = "WindowsTerminal.exe"
```

| Field | Required | Meaning |
| --- | --- | --- |
| `command` | yes | Program to start: a name the system finds on its own (on `PATH` or registered with the system, like `msedge.exe`), a full path, or a URI such as `ms-settings:`. Environment variables are expanded the way the system's shell does. |
| `args` | no | Arguments, one string each. Passed on as they are, quoted by the platform as needed. |
| `key`, `modifiers` | yes | As in `[[keybinding]]`. |
| `focus` | no | `true` focuses an open window of the application instead of starting it. Default `false`: start a new one every time. |
| `match_path`, `match_class`, `match_title` | no | Which windows count as the application's, with the same meaning as in window rules. Only read with `focus = true`. With none of them, a window counts when the file name of its executable, without extension, equals the file name of `command` without extension, ignoring case. |

## Design

### Configuration model (`mosaico-core/src/config/launch.rs`)

- `LaunchBinding { command, args, key, modifiers, focus, matcher }` and
  `WindowMatch { class, title, path }`.
- `KeybindingsFile` gains `#[serde(default)] launch: Vec<LaunchBinding>`.
  `Config`-level loading returns both lists.
- Validation: an empty `command` is an error for that entry, which is skipped
  with a warning like a bad key is today. `args` must be strings.
- `WindowMatch::matches(&WindowFacts)` reuses the rule matching in `rules.rs`
  (the matching moves to a shared function both call), plus the default
  file-stem comparison when no field is given.
- Conflicts are checked across `[[keybinding]]` and `[[launch]]` together, by
  `keybinding_conflicts.rs`, `mosaico doctor` and the settings app.
- Auto-merge of default keybindings leaves `[[launch]]` tables as they are.

### Launch contract (`mosaico-core/src/system/launcher.rs`)

```rust
/// Starts applications the user asked for by name.
pub trait Launcher {
    fn launch(&self, command: &str, args: &[String]) -> Result<()>;
}
```

Contract: start the program the way the system's own "run" box would find it,
detached from the daemon (no shared console or handles; it outlives the daemon),
in the user's home folder, without waiting for it. Fail with
`Error::Platform` when nothing by that name can be started.

Windows: `ShellExecuteExW` with `SEE_MASK_NOASYNC | SEE_MASK_DOENVSUBST |
SEE_MASK_FLAG_NO_UI`, `lpDirectory` set to the profile folder, and arguments
quoted with the rules `CommandLineToArgvW` reads.
ShellExecute resolves App Paths (`msedge.exe`), `PATH`, and URIs, and does not
inherit the daemon's handles, which matters since the daemon's handles
already caused a hang with msiexec.

### Hotkeys

- `hotkey::Target { Action(Action), Launch(usize) }`, where the index points
  into the loaded launch bindings. `Hotkeys::register` takes both lists; ids
  stay "position + 1" over the combined list. `triggered` returns a `Target`.
- `Rejected` names a launch binding by its command, so the daemon's startup
  warning reads "alt+shift+b (launch msedge.exe): already in use".
- Pausing unregisters launch bindings too; only `toggle-pause` stays, as now.
- The Windows hotkey thread sends `Target` down its channel instead of `Action`.

### Daemon

`daemon_loop_commands.rs` handles `Target::Launch(i)` beside `TogglePause`:

1. With `focus = true`, ask the tiling manager to focus a managed window that
   matches (`TilingManager::focus_matching(&WindowMatch) -> bool`).
2. Otherwise, or when nothing matched, call `Launcher::launch`. A failure is
   logged; it never stops the daemon.

The new window, once shown, is taken in by the existing new-window path and
lands on the focused monitor's active workspace.

### Focusing an open window (`mosaico-core/src/tiler/raise.rs`)

- Candidates are the windows the tiler manages, tiled or floating, on every
  monitor and workspace, read with `Desktop::facts` so windows on hidden
  workspaces are judged too.
- When the focused window matches, the next match in (monitor, workspace,
  position) order is chosen, wrapping, so pressing the key again cycles through
  the application's windows. Otherwise the first match.
- A window on a workspace out of view switches that monitor to its workspace,
  as when the user picks a hidden window from the taskbar, then focuses it. A
  window hidden by monocle leaves monocle.
- Windows the tiler does not manage (ignored by rules) are not candidates; the
  key then starts a new instance. See Decisions.

### Settings app

The Keybindings page gets an "Applications" section under the action rows:
one row per `[[launch]]` table with the shortcut capture, command, arguments
(one text field, split on spaces with quotes respected), the focus checkbox
and match path, plus Add and Remove. `KeybindingsDocument` gains positional
methods for `[[launch]]` (`launches`, `add_launch`, `set_launch_*`,
`remove_launch`) that keep comments, as the `[[keybinding]]` ones do.

## Decisions

Made without the maintainer while #207's syntax was still open. Each can be
revisited in review.

1. **`[[launch]]` tables, not `[[keybinding]] action = "launch"`.** The issue
   comment sketched the latter. A separate table keeps `Action` a `Copy` name,
   leaves auto-merge, the settings action rows and their file positions alone,
   and, most of all, an older daemon ignores an unknown table, while an unknown
   action fails the whole file and resets every binding to the defaults.
2. **`args` is an array of strings**, not one string. No quoting rules for
   users to learn, and nothing to get wrong with paths that hold spaces.
3. **Start a new instance by default; `focus = true` per binding** opts into
   focusing an open window. A terminal key usually wants a new window; a
   browser key usually wants the open one. Neither default fits both, and a new
   instance is what the user literally asked for.
4. **Matching reuses window rule fields** (`match_path`, `match_class`,
   `match_title`) so there is nothing new to learn, with a default based on the
   command's file name so the common case needs no extra lines. The default
   compares names without extension, so it also works on systems without them.
5. **Only managed windows are focused.** The tiler knows where they are and
   how to bring them into view across workspaces and monitors; a window it
   ignores may be a tool window or a splash screen.
6. **Pressing again cycles** through the application's windows, which is what
   other "run or raise" tools do and costs nothing extra.
7. **No default launch bindings.** Examples go in the template's comments and
   the docs.
8. **`ShellExecuteEx`, not `CreateProcess`,** on Windows, so App Paths names
   like `msedge.exe`, `PATH` lookups and URIs all work as in the Run box.
9. **Working folder is the home folder**, so a terminal opens there rather
   than in the daemon's folder.
10. **Not in this phase:** a `mosaico launch` CLI command (the shell can start
    programs already), launch from bar clicks, and hot-reloading keybindings,
    which still need a daemon restart as before.
11. **Settings app support is in this phase,** so the feature is usable
    without editing TOML, as phase 37 set out.

## Tasks

Each task is one pull request, stacked on the one before.

### 38.1 Plan

- [x] This file and the roadmap row

### 38.2 Launch contract

- [ ] `system::Launcher` trait with its contract, and a recording fake for tests
- [ ] Windows `ShellLauncher`: `ShellExecuteExW`, argument quoting, home folder,
      with tests for quoting and for a command that does not exist

### 38.3 Configuration

- [ ] `LaunchBinding` and `WindowMatch`, parsed from `[[launch]]`, with tests
      for every field, defaults, an empty command and an old file without the
      table
- [ ] Rule matching shared between window rules and `WindowMatch`, plus the
      file-stem default
- [ ] Conflicts across both tables; `mosaico doctor` checks launch keys and
      conflicts
- [ ] Commented examples in the keybindings template; auto-merge keeps
      `[[launch]]`

### 38.4 Hotkeys and daemon

- [ ] `hotkey::Target`; registration, pause and rejections over both lists
- [ ] Windows hotkey channel carries `Target`; the daemon starts the program
- [ ] Metrics baselines blessed if they move

### 38.5 Focus an open window

- [ ] `TilingManager::focus_matching`: first match, cycling, hidden workspace,
      other monitor, monocle, no match, with tests
- [ ] Daemon falls back to launching when nothing matched

### 38.6 Settings app

- [ ] `KeybindingsDocument` methods for `[[launch]]`, keeping comments
- [ ] "Applications" section: add, edit, remove, conflicts, with app tests

### 38.7 Documentation and verification

- [ ] `website/src/guide/keybindings.md`: "Launching Applications" section
- [ ] Manual: a launch key starts Edge and Windows Terminal; with `focus`, the
      key brings Edge back from another workspace and cycles its windows; a
      missing program logs an error and the daemon keeps running

[`Action`]: ../crates/mosaico-core/src/action.rs

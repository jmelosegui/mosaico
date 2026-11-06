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

Windows: `ShellExecuteExW` with `SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI`,
`lpDirectory` set to the profile folder, and arguments quoted with the rules
`CommandLineToArgvW` reads. `%VAR%` references are expanded in the command and
the arguments. The shell guesses a web address from a name it cannot find and
opens it in the browser, so the launcher first looks the name up itself: a URI,
a path that exists, a name on `PATH` (trying `PATHEXT` for one without an
extension), or an App Paths key. Anything else fails with an error.
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

The hotkey bridge turns `Target::Launch(i)` into a daemon message, and
`daemon_loop_commands::launch` hands binding `i` and the platform's launcher
to `TilingManager::launch(&LaunchBinding, &dyn Launcher)` in core, which
decides:

1. With `focus = true`, focus a managed window that matches, if there is one.
   With `focus` off, no window is looked at.
2. Otherwise, or when nothing matched, call `Launcher::launch`. A failure is
   logged and shown beside the tray icon (decision 24); it never stops the
   daemon.

Keeping the decision in core lets it be tested with the fake platform and a
recording launcher.

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

The Keybindings page splits into two tabs, the way the Bar page does:
**Actions**, the fixed actions of mosaico whose shortcuts alone change, and
**Applications**, one row per `[[launch]]` table with the shortcut capture,
command, arguments
(one text field, split on spaces with quotes respected), the focus checkbox
and match path, plus Add and Remove. `KeybindingsDocument` gains positional
methods for `[[launch]]` (`launches`, `add_launch`, `set_launch`,
`set_launch_key`, `remove_launch`) that keep comments, as the `[[keybinding]]`
ones do.

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
   focusing an open window. Without it, the daemon never looks for a matching
   window: every press starts the program again, and an application that
   allows only one instance decides itself what a second start does. A
   terminal key usually wants a new window; a browser key usually wants the
   open one. Neither default fits both, and a new instance is what the user
   literally asked for.
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
    programs already) and launch from bar clicks. Hot-reloading keybindings
    was here too, until the manual test showed its cost (decision 21).
11. **Settings app support is in this phase,** so the feature is usable
    without editing TOML, as phase 37 set out.
12. **A name the system cannot find is an error, not a web search.** The shell
    turns an unknown name into a web address and opens the browser, so the
    Windows launcher looks the name up first, the way the shell does.
13. **The launch runs on the daemon's thread** and waits only for the shell to
    hand the program over, not for its window. It is a key press, rare and
    short, and a failure can then be returned to the caller and logged.
14. **The settings app edits command, arguments, match path and focus.** Class
    and title matching stay in the file, kept as written, but get no fields:
    the path covers the common case and three more fields would crowd the
    editor.
15. **Arguments are one text field,** split on spaces, with double quotes
    grouping and `""` an empty argument. The file still stores an array.
16. **A new application starts blank with its editor open,** and the shortcut
    is captured only when its button is clicked, so typing the command is
    never taken as a key press. Apply refuses a launch binding with no command
    or no shortcut, including one written by hand, and says which.
17. **Replacing a taken shortcut removes the launch binding that had it,**
    as it removes an action binding that had it: a launch binding without a
    shortcut does nothing.
18. **Restore defaults leaves launch bindings alone.** There are no default
    ones, and removing the user's would be a surprise.
19. **An open editor survives the window rereading the file.** The window
    follows the file every two seconds while nothing is edited; the editor
    stays open and starts over from the file only when it changed under it.
20. **Applications get a tab of the Keybindings page** (the maintainer's
    call, replacing a section under the actions). Actions are fixed and only
    their keys change; applications are added and removed, so they get a
    view of their own, with a tab as on the Bar page rather than another
    sidebar entry. The filter and Restore defaults belong to Actions; a
    search of the window shows matches from both tabs, and a key press is
    taken only by a row waiting on the tab shown.
21. **`keybindings.toml` is hot-reloaded.** An application added in the
    settings app was saved, and its shortcut then did nothing, with nothing
    saying a restart was needed. The daemon now reloads the file when it
    changes: a `HotkeyListener` in core stops the hotkey source and starts
    it with the new bindings, paused if the hotkeys were. Stopping first
    frees the keys the new bindings may reuse. A press sent by the old
    source just before it stopped is read against the new launch bindings;
    that window is a few milliseconds and was not worth a generation number
    on every press. Missing default actions are still appended only at
    startup, so a reload never writes the file the settings app is saving.
22. **The editor shows the shortcut and says what each field is for.** A
    maintainer adding an application found no way to set its shortcut: only
    the row's unlabelled pencil captured it. The editor now starts with the
    shortcut and a **Record** button, a row without one shows **Set
    shortcut**, and every field has a line of help under it. Capture still
    starts only from a click (decision 16).
23. **The windows' path is asked for only when focusing.** Nothing else
    uses it, so it sits under the focus checkbox and appears when that is
    ticked. A path already in the file is kept while it is hidden.
24. **A program that cannot be started is told beside the tray icon.** A
    press of a launch key has nobody waiting for its answer, and logging is
    off by default, so a failure that was only logged left the user with a
    key that did nothing. `Tray::report_launch` in core logs every outcome
    and, on a failure, shows a notice naming the command and the reason.
    A program that starts, or a window focused, shows nothing: the window
    is the answer. Without the tray icon the failure is only logged; a
    message box was rejected, as it would take focus from the window the
    user was in.

## Tasks

Each task is one pull request, stacked on the one before.

### 38.1 Plan

- [x] This file and the roadmap row

### 38.2 Launch contract

- [x] `system::Launcher` trait with its contract
- [x] Windows `ShellLauncher`: `ShellExecuteExW`, argument quoting, variables,
      home folder, with tests for quoting and for a command that does not exist
- [x] Names looked up as the shell does (URI, path, `PATH` with `PATHEXT`, App
      Paths) before the shell is asked, because the shell turns a name it cannot
      find into a web search in the browser

### 38.3 Configuration

- [x] `LaunchBinding` and `WindowMatch`, parsed from `[[launch]]`, with tests
      for every field, defaults, an empty command and an old file without the
      table
- [x] Rule matching shared between window rules and `WindowMatch`, plus the
      file-stem default
- [x] Conflicts across both tables; `mosaico doctor` checks launch keys and
      conflicts
- [x] Commented examples in the keybindings template; auto-merge keeps
      `[[launch]]`

### 38.4 Hotkeys and daemon

- [x] `hotkey::Target`; registration, pause and rejections over both lists
- [x] A recording fake `Launcher` for the daemon's tests
- [x] Windows hotkey channel carries `Target`; the daemon starts the program
- [x] A program that cannot be started shows a tray notice, not only a log line
- [x] Metrics baselines blessed if they move

### 38.5 Focus an open window

- [x] `Input::Raise` and `Tiler::raise`: hidden workspace, other monitor, monocle
- [x] `TilingManager::launch` with `focus`: first match, cycling, falls back to
      launching when nothing matched, with tests

### 38.6 Settings app

- [x] `KeybindingsDocument` methods for `[[launch]]`, keeping comments
- [x] "Applications" section: add, edit, remove, conflicts, with app tests
- [x] Actions and Applications as tabs of the Keybindings page
- [x] `keybindings.toml` hot-reloaded, so a saved shortcut works at once
- [x] Application editor: shortcut line, Set shortcut, help, path only when focusing

### 38.7 Documentation and verification

- [x] `website/src/guide/keybindings.md`: "Launching Applications" section,
      and the Applications section in `website/src/guide/settings.md`
- [x] Manual: a launch key starts Edge and Windows Terminal; with `focus`, the
      key brings Edge back from another workspace and cycles its windows; a
      missing program logs an error and the daemon keeps running. Checked on
      the live desktop with a separate instance: without `focus` every press
      opened a new Terminal window; with `focus` presses cycled through three
      Terminal windows and two Edge windows; Edge (not on `PATH`) was found
      through App Paths; it was brought back from a hidden workspace, from the
      other monitor and from behind a monocle window; a missing program logged
      a warning and opened nothing. The check found that a press which only
      focused a window was logged as "started"; `launch` now returns
      `Launched`, and the log says "focused an open window".

[`Action`]: ../crates/mosaico-core/src/action.rs

# Phase 26: Floating Windows

**Status:** In progress

**Goal:** Let a managed window leave the layout and float above the tiles of
its workspace, from a key or from a window rule, without becoming unmanaged.

## Overview

A window rule can only tile a window or ignore it. An ignored window gets no
border, is not hidden when its workspace is switched away, and stays on
screen over every workspace. Small tools, pickers, players and the
mosaico settings window do not belong in the grid, but they still belong to a
workspace.

A floating window is managed but not tiled:

- It belongs to one workspace on one monitor, is hidden and shown with it, and
  moves with `send-to-workspace`.
- The layout never moves or resizes it. The user moves and resizes it, and
  can drag it to another monitor, where it joins the workspace on screen.
- It stays above the tiles of its workspace, without being always on top of
  other applications.
- It gets the focused border while it has focus, and the configured corners.
- It is shown again when the daemon stops, and after a crash, like any window
  the daemon hid.

## Configuration

### Rules: one `action` per rule

A rule says what happens to the windows it matches with one field, so a rule
cannot contradict itself, and the first matching rule decides everything:

```toml
[[rule]]
match_path = "calc.exe"
action = "float"      # managed, floating

[[rule]]
match_class = "SomeOverlay"
action = "ignore"     # never managed

[[rule]]
match_path = "photocraft.exe"
action = "tile"       # tiled, even without a standard title bar
```

`manage = true` and `manage = false` keep working, as `"tile"` and
`"ignore"`, so existing `user-rules.toml` files and the community rules need
no change. A rule with both fields, or with neither, is an error.

A window without a standard title bar is only managed when a rule matches it
with `"tile"` or `"float"`. Owned windows, tool windows and dialogs are never
managed.

The settings window floats by a rule built into mosaico, evaluated after the
user rules and before the community rules, so a user rule can still tile it.

### Keybindings

```toml
[[keybinding]]
action = "toggle-float"   # Alt+F
key = "F"
modifiers = ["alt"]

[[keybinding]]
action = "cycle-float"    # Alt+Shift+F
key = "F"
modifiers = ["alt", "shift"]
```

Both are also CLI actions (`mosaico action toggle-float`).

Resizing a floating window from the keyboard, centered on the window:
`grow-width`/`shrink-width` on Alt+Period/Alt+Comma, and
`grow-height`/`shrink-height` on Alt+Shift+Period/Alt+Shift+Comma (`>` and
`<`). Plus and Minus are avoided: with Alt+Shift, terminals split panes.

## Behaviour

### Toggling

- **Tiled to floating.** The window leaves the layout, which closes the gap.
  It goes back to the floating rect it had last time, in this session, moved
  into the work area of its monitor when it no longer fits. A window floating
  for the first time is centered on its monitor at two-thirds of the work
  area's width and height. It keeps focus and comes to the front.
- **Floating to tiled.** Its rect is remembered for the next toggle, and it
  joins the end of the layout and keeps focus.
- Toggling in monocle leaves monocle first.

### Windows a rule floats

A new window a rule floats joins the floating windows of the workspace on
screen of the focused monitor, as a new tiled window would join its layout.
It keeps the size its application chose and is centered in the work area,
shrunk to fit when it is larger.

A rules reload that changes a managed window between `"tile"` and
`"float"` moves it at once, as toggling does. One the new rules ignore is
let go, as before.

### Stacking

On the workspace on screen, every floating window stays above every tile,
without being always on top. The tiler keeps one stacking order of floating
windows per workspace, back to front. Whenever a tile comes to the front,
focused by a key, by the user or by the system, the tiler puts it behind the
backmost floating window. A floating window that gets focus moves to the front
of that order. This one rule, applied where focus changes, replaces raising
windows at scattered call sites.

A window that becomes floating, toggled or by a rules reload, comes in front
of the other floating windows at once, even though it already had focus. The
border of the focused tile goes behind the backmost floating window rather
than behind its tile, so it never draws over a floating window.

The user's retile tiles again every floating window on the workspaces on
screen, except those the rules float. `toggle-float` does not tile those
either: the rule wins until the user changes it.

### Focus and navigation

While a workspace has floating windows, outside monocle, its tiles are out of
reach of the keys, as in monocle. This keeps the tiler from focusing a tile,
which would bring it in front of the floating windows before it can be put
behind them again.

- `focus-left/right` go to the adjacent monitor, from a floating window or a
  tile the user clicked. `focus-up/down` do nothing.
- `cycle-float` focuses the next floating window of the workspace on screen,
  backmost first, so repeating it visits every one. From a tile it focuses the
  frontmost floating window.
- `move-left/right/up/down` on a floating window nudges it by a step of
  one-twentieth of the work area's width or height, stopping at the edge of
  the work area. On a tile they do nothing.
- `grow-width`/`grow-height` move both edges of that axis of the focused
  floating window outward by half a step each, so it stays centered, stopping
  at the work area; `shrink-*` moves them inward, never below the window's
  minimum size. On a tile they do nothing.
- Coming back to a workspace or monitor focuses its frontmost floating window.
- A floating window stays on its workspace: `send-to-workspace` ignores it.

### Monocle

Floating windows are hidden while their workspace is in monocle, and shown
again when it leaves monocle. Focusing a hidden floating window, as from the
taskbar, leaves monocle.

### Monitors

A floating window moves to another monitor in two ways: the user drags it
there, or its monitor disconnects. Both go through one
path, which keeps its position relative to the work area: a window at the
left third of one monitor lands at the left third of the other, shrunk to fit.
A window displaced by a disconnect goes back when its monitor returns, if it
is still where it was put. After a display change, a floating window that no
longer fits its monitor's work area is moved into it.

### Removal

A floating window leaves its workspace when it closes, hides or is minimized,
like a tile. Focus goes to the next floating window, or when none is left to
the window the workspace remembers, or its first tile. Tiling a floating
window again hands focus on the same way, keeping it on the window when no
other floats.

## Architecture

- `Workspace` gains `floating: Vec<WindowId>`, back to front. `windows()`,
  `contains()` and `index_of()` stay about tiles, so the layout, drag and
  monocle code is unchanged. `holds()` and `floating()` cover floating windows.
- `Tiler` gains `float_rects: HashMap<WindowId, Rect>`, the last floating rect
  of each window, forgotten when the window is destroyed.
- `config`: `RuleAction { Tile, Float, Ignore }`; `matching_rule` returns
  the first matching rule, and `config::decide` adds the title bar check.
  `Tiler::rules_decide(facts) -> RuleAction` uses it, and `mosaico debug list`
  shows the same decision.
- Lookups: `find_window` stays tiled-only, `find_floating` finds floating
  windows, `find_managed` either.
- No new effect: placing uses `Effect::Place`, stacking uses
  `Effect::PutBehind`, and hiding uses `Effect::Hide`/`Show`.
- `DesktopSnapshot` lists the floating windows of each workspace for
  `mosaico debug state`.

## Tasks

- [x] 26.1 Rules: `action = "tile" | "float" | "ignore"`, `manage` still read
- [x] 26.2 `Workspace` floating windows and lookups
- [x] 26.3 `toggle-float` and `cycle-float` actions, CLI, default keys
- [x] 26.4 Placement: remembered rect, centered two-thirds, clamped to work area
- [x] 26.5 Adoption by rule, and rules reload between tile and float
- [x] 26.6 Workspace switch, send-to-workspace and monocle hide floating windows
- [x] 26.7 Stacking: floating windows stay above the tiles of their workspace
- [x] 26.8 Focus, navigation, removal and borders
- [x] 26.8b Keyboard nudge, `grow-width/height` and `shrink-width/height`
- [x] 26.9 Monitors: drag, move, disconnect and reconnect, display changes
- [x] 26.10 Shutdown and crash recovery show floating windows
- [x] 26.11 `debug state` lists floating windows
- [x] 26.12 Metric scenarios for toggling and switching with floating windows
- [x] 26.13 Docs: window rules, keybindings, configuration, website guide
- [ ] 26.14 Manual verification on the live desktop

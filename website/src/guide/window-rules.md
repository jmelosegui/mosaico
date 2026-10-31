# Window Rules

Window rules decide which windows Mosaico tiles, which it floats above the
tiles, and which it leaves alone. Rules are split into two files:

| File | Purpose |
|------|---------|
| `rules.toml` | Community rules, downloaded on daemon startup |
| `user-rules.toml` | Your personal rules, never overwritten |

Both files live in `~/.config/mosaico/`.

## How It Works

When the daemon starts, it downloads the latest community rules from the
[mosaico-rules](https://github.com/jmelosegui/mosaico-rules) repository
and saves them to `rules.toml`. Your personal rules in `user-rules.toml`
are never touched.

At runtime, both files are merged: **user rules are evaluated first**, so
they take priority over community defaults. The first matching rule wins.

## Rule Format

Each rule is a `[[rule]]` entry:

```toml
[[rule]]
match_class = "ApplicationFrameWindow"
action = "ignore"

[[rule]]
match_title = "Calculator"
action = "float"

[[rule]]
match_class = "Chrome_WidgetWin_1"
match_title = "Picture-in-Picture"
action = "float"

[[rule]]
match_path = "photocraft.exe"
action = "tile"
```

## Rule Fields

| Field | Type | Description |
|-------|------|-------------|
| `match_class` | string | Match by window class name (case-insensitive, exact) |
| `match_title` | string | Match by window title (case-insensitive, substring) |
| `match_path` | string | Match by the program's executable path (case-insensitive, ends with) |
| `action` | string | What to do with the window: `"tile"`, `"float"` or `"ignore"` |

Every rule needs an `action`. Rules written before `action` existed use
`manage = true`, which means `action = "tile"`, or `manage = false`, which
means `action = "ignore"`. Both still work, but a rule cannot have both
`action` and `manage`.

| Action | What happens |
|--------|--------------|
| `tile` | The window is tiled into the layout |
| `float` | The window is managed but not tiled: it keeps its own size and stays above the tiles of its workspace, and moves with the workspace. `toggle-float` and `retile` leave it floating |
| `ignore` | Mosaico leaves the window alone |

## Matching Behavior

- **Class match**: case-insensitive **exact** match against the window
  class name.
- **Title match**: case-insensitive **substring** match against the window
  title.
- **Path match**: case-insensitive match against the end of the full path
  of the program that owns the window. The match must start at a folder
  boundary, so `"app.exe"` matches `C:\Program Files\Vendor\app.exe` but
  not `C:\Tools\myapp.exe`. You can give just the file name, part of the
  path (`"Vendor\app.exe"`), or the full path. Forward and back slashes
  are treated the same.
- If a rule specifies more than one field, **all** of them must match.
- A rule must specify at least one field. A rule with none matches
  nothing.

Prefer `match_path` when an app uses a generic class name shared with
other programs, or when its title changes with the open document.

## Apps With Custom Title Bars

Some apps hide the standard Windows title bar and draw their own. To
Windows these look the same as splash screens and borderless popups, so
Mosaico does not manage them unless a rule opts them in with
`action = "tile"` or `action = "float"`:

```toml
# user-rules.toml
[[rule]]
match_path = "photocraft.exe"
action = "tile"
```

Owned windows, tool windows, and dialogs are never tiled, even when a
rule matches them, so a rule like this does not pull in the app's
dialogs.

## Evaluation Order

Rules are evaluated in order. User rules (`user-rules.toml`) come first,
followed by community rules (`rules.toml`). The **first matching rule
wins**. If no rule matches a window, it is tiled.

This means you can override any community rule by adding a rule in
`user-rules.toml`. For example, to tile a window that community rules
ignore:

```toml
# user-rules.toml
[[rule]]
match_class = "Chrome_WidgetWin_1"
action = "tile"
```

## Community Rules

Community rules are maintained at
[github.com/jmelosegui/mosaico-rules](https://github.com/jmelosegui/mosaico-rules).
They cover common exclusions like UWP apps, GPU overlays, system dialogs,
and VPN clients.

If you find a window that should be excluded for all users, consider
[contributing](https://github.com/jmelosegui/mosaico-rules/blob/main/CONTRIBUTING.md)
the rule upstream instead of keeping it in `user-rules.toml`.

## Hot-Reload

Changes to `user-rules.toml` are automatically detected and applied while
the daemon is running. When the file changes, both rule sets are re-merged,
your rules first. New rules apply to new windows. Managed windows the new
rules no longer manage are released at once, and their monitor is retiled.
A window the new rules would manage is picked up the next time it gets
focus.

A `user-rules.toml` that does not parse counts as no user rules, so the
community rules apply alone until it is fixed.

Community rules (`rules.toml`) are only updated on daemon startup.

## Finding Window Class Names and Paths

Use the `debug list` command to see the class names and executable paths
of all visible windows:

```sh
mosaico debug list
```

The output includes the class name and path for each window, which you
can use in your rules.

# Window Rules

Window rules control which applications Mosaico manages. Rules are split
into two files:

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
manage = false

[[rule]]
match_title = "Calculator"
manage = false

[[rule]]
match_class = "Chrome_WidgetWin_1"
match_title = "Picture-in-Picture"
manage = false
```

## Rule Fields

| Field | Type | Description |
|-------|------|-------------|
| `match_class` | string | Match by window class name (case-insensitive, exact) |
| `match_title` | string | Match by window title (case-insensitive, substring) |
| `manage` | bool | Whether to tile the window (`true` or `false`) |

## Matching Behavior

- **Class match**: case-insensitive **exact** match against the window
  class name.
- **Title match**: case-insensitive **substring** match against the window
  title.
- If a rule specifies both `match_class` and `match_title`, **both** must
  match.
- If a rule specifies neither, it matches everything.

## Evaluation Order

Rules are evaluated in order. User rules (`user-rules.toml`) come first,
followed by community rules (`rules.toml`). The **first matching rule
wins**. If no rule matches a window, it is managed by default.

This means you can override any community rule by adding a rule in
`user-rules.toml`. For example, to force-tile a window that community
rules exclude:

```toml
# user-rules.toml
[[rule]]
match_class = "Chrome_WidgetWin_1"
manage = true
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
the daemon is running. When the file changes, both rule sets are re-merged
and existing windows are re-evaluated against the new rules.

Community rules (`rules.toml`) are only updated on daemon startup.

## Finding Window Class Names

Use the `debug list` command to see the class names of all visible windows:

```sh
mosaico debug list
```

The output includes the class name for each window, which you can use in
your rules.

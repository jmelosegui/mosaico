# Window Rules

Window rules control which applications Mosaico manages. Rules are defined
in `~/.config/mosaico/rules.toml`.

## Configuration

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

Rules are evaluated in the order they appear in the file. The **first
matching rule wins**. If no rule matches a window, it is managed by default.

This means you should place more specific rules before general ones.

## Default Rules

Even without a `rules.toml` file, Mosaico excludes:

- `ApplicationFrameWindow` -- UWP apps like Settings and Microsoft Store

## Hot-Reload

Changes to `rules.toml` are automatically detected and applied while the
daemon is running. New rules apply to newly opened windows; existing managed
windows are not re-evaluated.

## Finding Window Class Names

Use the `debug list` command to see the class names of all visible windows:

```sh
mosaico debug list
```

The output includes the class name for each window, which you can use in
your rules.

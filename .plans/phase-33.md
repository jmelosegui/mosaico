# Phase 33: Theme Accent Override

**Status:** Pending

**Goal:** Let users pick a custom accent color from the active theme's
palette without having to override every individual color slot in
`[bar.colors]` and `[borders.colors]`.

## Overview

Phase 32 introduced three theme families (Catppuccin, Rosé Pine, Tokyo
Night) each with multiple flavors. Each flavor pins one specific
palette color to the "lead" role: Iris for Rosé Pine, Blue for
Catppuccin and Tokyo Night. A user who likes a flavor's overall
palette but wants a different accent currently has to override four
slots manually:

```toml
[bar.colors]
foreground = "gold"
inactive_workspace = "gold"
pill_border = "gold"

[borders.colors]
focused = "gold"
```

This phase adds a single config field that does the same thing in
one line:

```toml
[theme]
name = "rose-pine"
flavor = "main"
accent = "gold"     # NEW: replaces Iris in all four "lead" slots
```

`accent` resolves through the active palette like any other named
color, so the same field works under any theme: `accent = "mauve"`
under Catppuccin Mocha, `accent = "purple"` under Tokyo Night, etc.

## Configuration

`ThemeConfig` gains an optional third field:

```rust
pub struct ThemeConfig {
    pub name: String,
    pub flavor: String,
    pub accent: String,   // empty = family default; resolves via palette
}
```

`accent` is `#[serde(default)]` so missing fields stay backward
compatible.

## Slots Affected

When `accent` resolves to a non-empty hex, it replaces the lead color
in:

1. `bar.foreground`
2. `bar.inactive_workspace`
3. `bar.pill_border`
4. `border_focused`

Slots intentionally left alone (theme identity should still read as
the chosen flavor):

- `bar.background` -- the theme's base background
- `bar.accent` -- the secondary accent slot stays on the flavor's
  default (e.g. Rose under Rosé Pine, Green under Catppuccin)
- `bar.widget_background` -- theme-specific surface
- `border_monocle` -- monocle has its own meaning; stays Green
- `border_unfocused` -- theme-specific muted gray; stays Subtle/Comment

Net effect: `accent = "gold"` under Rosé Pine Main produces a
recognizably Rosé Pine bar (Base background, Rose secondary, Foam
monocle, Subtle unfocused) with a Gold lead instead of Iris.

## Override Precedence

User overrides in `[bar.colors]` or `[borders.colors]` still win over
the accent field because they are more specific:

```toml
[theme]
name = "rose-pine"
accent = "gold"

[borders.colors]
focused = "love"      # this wins; focused border is Love, not Gold
```

The order of resolution becomes:

1. Explicit hex / named color in `[bar.colors]` / `[borders.colors]`
2. `[theme] accent` if set (and the slot is one of the four affected)
3. Flavor's default

## Architecture

### Schema

```rust
// ThemeConfig
pub struct ThemeConfig {
    pub name: String,
    pub flavor: String,
    pub accent: String,
}
```

### Resolution

`Config::validate()` adds an accent-resolution step that runs after
`resolve_borders()` and before the bar config is finalized:

```rust
fn resolve_accent(&mut self) {
    if self.theme.accent.is_empty() {
        return;
    }
    let theme = self.theme.resolve();
    let Some(hex) = theme.named_color(&self.theme.accent) else {
        eprintln!(
            "Warning: unknown accent {:?} for theme {:?}; ignored",
            self.theme.accent, self.theme.name
        );
        return;
    };
    // Override the four "lead" slots only when the user did not set
    // them explicitly (empty string means "use theme default").
    if self.borders.colors.focused == theme.border_focused() {
        self.borders.colors.focused = hex.to_string();
    }
    // ... same for the three bar slots, gated on "still equal to
    // the theme default" so explicit overrides win.
}
```

The "still equal to the theme default" check is how we keep explicit
user overrides winning. It compares the post-`resolve_borders` value
to the flavor's canonical default; if the user set their own value,
the post-resolve string differs and the accent skips that slot.

### Bar resolution

`BarConfig::resolve_colors()` already takes a `Theme`. We extend it
to optionally take the resolved accent hex and apply the same
"only-if-still-default" logic for `foreground`, `inactive_workspace`,
and `pill_border`.

## Modified Files

```
crates/mosaico-core/src/config/
  theme.rs            # Add accent field to ThemeConfig
  mod.rs              # Add resolve_accent() in Config::validate
  bar.rs              # Thread accent through resolve_colors
  template_config.rs  # Document accent in the [theme] block
crates/mosaico-windows/src/
  daemon_loop_handlers.rs  # Pass accent through on hot-reload
```

## Edge Cases

1. **Unknown accent name**: log a warning and ignore (fall back to
   flavor default).
2. **Empty accent**: no-op, current behavior preserved.
3. **Hex accent**: `accent = "#abc123"` should pass through directly
   without going through the palette (matches how the existing
   per-slot color fields work).
4. **Accent collides with secondary slot**: if `accent = "rose"`
   under Rosé Pine, both `bar.foreground` and `bar.accent` end up
   as Rose. Acceptable; the user asked for it.
5. **Hot-reload**: editing `accent` should re-resolve colors live,
   same as editing `flavor`.

## Tasks

- [ ] Add `accent: String` to `ThemeConfig` (default empty)
- [ ] Add `Config::resolve_accent()` running after
      `resolve_borders()`; apply the "only-if-still-default" rule
- [ ] Thread the resolved accent into `BarConfig::resolve_colors()`
- [ ] Document `accent` in the `mosaico init` template `[theme]` block
- [ ] Update `docs/configuration.md` and `docs/theming.md`
- [ ] Update `website/src/guide/configuration.md` and
      `website/src/guide/theming.md`
- [ ] Add unit tests:
  - accent overrides the four slots when set
  - explicit `[borders.colors] focused = "..."` wins over accent
  - unknown accent name is ignored with a warning
  - hex accent (`accent = "#abc123"`) passes through
  - hot-reload picks up new accent without restart
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets -- -D warnings`
- [ ] Run `cargo test --lib`
- [ ] Manual test: set `accent = "gold"` under Rosé Pine, confirm
      bar foreground + focused border turn gold while Rose remains
      the secondary accent
- [ ] Manual test: set `accent = "mauve"` under Catppuccin Mocha,
      confirm bar lead turns mauve
- [ ] Manual test: set `accent = "purple"` under Tokyo Night, confirm
      similar
- [ ] Manual test: set both `accent = "gold"` and
      `[borders.colors] focused = "love"`, confirm the explicit
      override wins for the focused border but the bar still reads gold
- [ ] Update `.plans/plan.md`

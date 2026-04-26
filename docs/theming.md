# Theming

Mosaico includes a built-in Catppuccin theme system that applies a consistent
color scheme across all visual elements -- focus borders, monocle borders,
and the status bar -- from a single configuration line.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/config/theme.rs` | `ThemeConfig`, `Theme` enum, color resolution |
| `crates/mosaico-core/src/config/palette.rs` | Hardcoded hex values for all 4 flavors |
| `crates/mosaico-core/src/config/mod.rs` | `Config::validate()` integrates theme resolution |
| `crates/mosaico-core/src/config/bar.rs` | `BarConfig::resolve_colors()` uses theme |

### Key Types

- `ThemeConfig` -- user-facing config: `name: String` (default `"catppuccin"`),
  `flavor: String` (default `"mocha"`)
- `Theme` -- resolved enum, one variant per family carrying its
  flavor enum: `Catppuccin(CatppuccinFlavor)`, `RosePine(RosePineFlavor)`,
  `TokyoNight(TokyoNightFlavor)`. The nesting keeps invalid
  combinations (e.g. Rosé Pine with a Mocha flavor) unrepresentable
  at the type level
- `CatppuccinFlavor` -- `Mocha` (default), `Macchiato`, `Frappe`, `Latte`
- `RosePineFlavor` -- `Main` (default), `Moon`, `Dawn`
- `TokyoNightFlavor` -- `Night` (default), `Storm`, `Day`

## Theme Families

| Family | Flavors | Style |
|--------|---------|-------|
| Catppuccin | Latte / Frappé / Macchiato / Mocha | Light through darkest |
| Rosé Pine | Main / Moon / Dawn | Two darks, one light |
| Tokyo Night | Night / Storm / Day | Two darks, one light |

Each flavor has its own named-color table and bar color mapping,
defined in `crates/mosaico-core/src/config/palette/<family>.rs`.

### Common Color Aliases

To keep portable configs (`focused = "blue"`) working across themes,
every family exposes the four most common color names: `red`, `blue`,
`green`, `yellow`. The exact shade differs per theme so the highlight
reads well against that theme's base background.

Catppuccin additionally exposes 14 native accent colors (rosewater,
flamingo, pink, mauve, red, maroon, peach, yellow, green, teal, sky,
sapphire, blue, lavender). Rosé Pine exposes its six native names
(love, gold, rose, pine, foam, iris). Tokyo Night exposes red, orange,
yellow, green, cyan, teal, blue, purple plus aliases `mauve` and
`magenta` for `purple`.

## Named Color Resolution

Throughout the configuration, anywhere a hex color is expected, you can
instead write a Catppuccin color name:

```toml
# These are equivalent when using Mocha:
focused = "#89b4fa"
focused = "blue"
```

`Theme::resolve_color(value, fallback)` handles three cases:

1. **Empty string** -- returns the fallback (theme default)
2. **Starts with `#`** -- returned as-is (explicit hex)
3. **Otherwise** -- looked up as a named color in the active palette

This applies to border colors in `config.toml` and all color fields in
`bar.toml`.

## Color Mapping

### Border Colors

| UI Element | Theme Default |
|------------|---------------|
| Focused border | `Blue` |
| Monocle border | `Green` |

### Bar Colors

| UI Element | Mocha Default |
|------------|---------------|
| Background | `#1e1e2e` (Base) |
| Foreground | `#cdd6f4` (Text) |
| Active workspace | `#89b4fa` (Blue) |
| Active workspace text | `#1e1e2e` (Base) |
| Inactive workspace | `#45475a` (Surface1) |
| Separator | `#313244` (Surface0) |
| Accent | `#f38ba8` (Red) |
| Widget background | `#313244` (Surface0) |
| Pill border | `#45475a` (Surface1) |

Each flavor has its own complete bar color palette defined in
`palette/<family>.rs` (one file per theme family).

## Configuration

In `config.toml`:

```toml
[theme]
name = "catppuccin"   # catppuccin, rose-pine, tokyo-night
flavor = "mocha"
```

Recognized name + flavor combinations:

| name | flavor |
|------|--------|
| `catppuccin` | `mocha`, `macchiato`, `frappe`, `latte` |
| `rose-pine` (alias `rosepine`, `rose_pine`) | `main`, `moon`, `dawn` |
| `tokyo-night` (alias `tokyonight`, `tokyo_night`) | `night`, `storm`, `day` |

Both name and flavor are case-insensitive. Unknown values fall back
to the family default (e.g. `name = "rose-pine"` with an unknown
flavor resolves to Main); unknown family names fall back to Catppuccin
Mocha.

## Color Precedence

When a theme is active:

1. **Explicit hex in config** -- highest priority, always used
2. **Named color in config** -- resolved via the theme palette
3. **Empty/omitted field** -- filled from theme defaults
4. **Hardcoded app default** -- only if no theme is configured

This means existing configurations with explicit hex colors continue to
work unchanged after enabling a theme. The theme only fills in unspecified
fields.

## Integration

### Config Loading Pipeline

1. `load()` reads `config.toml` including the `[theme]` section
2. `Config::validate()` calls `theme.resolve()` to get the concrete `Theme`
3. `validate()` calls `resolve_borders()` which fills border colors from
   the theme
4. When loading `bar.toml`, `BarConfig::resolve_colors(theme)` fills bar
   colors from the theme

### Hot-Reload

The daemon tracks `current_theme`. On `ConfigReload::Config`:

1. Re-resolves the theme from the new config
2. If the theme changed, calls `bar_mgr.resolve_colors(new_theme)` to
   update bar colors without recreating bar windows
3. Passes updated border colors to `TilingManager::reload_config()`

### Doctor Integration

`mosaico doctor` validates that the theme section parses correctly as part
of the `config.toml` validation check.

## Design Decisions

- **Built-in palettes** rather than external theme files keep the
  implementation simple and dependency-free. All 56 color values (14 per
  flavor x 4 flavors) are compiled as constants.
- **Named color strings** rather than a separate override struct provide a
  natural configuration experience -- users write `"blue"` instead of
  learning a theme-specific override syntax.
- **Theme defaults fill gaps** rather than overriding explicit values,
  respecting user customization while providing a coherent default palette.
- **Mocha as default** because it is the most popular Catppuccin flavor and
  provides good contrast for a tiling window manager.
- **Bar colors per-flavor** rather than derived from accent colors, because
  each flavor's surface/background colors differ significantly and need
  individual tuning.

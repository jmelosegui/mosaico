# Phase 15: Catppuccin Theming

**Status:** Planned

**Goal:** Add a built-in theming system using the Catppuccin color palette.
A theme applies a consistent color scheme across all visual elements --
focus borders, monocle borders, and the status bar -- from a single
configuration line.

## Overview

Catppuccin provides four flavors ranging from light to dark:

| Flavor | Description |
|--------|-------------|
| Latte | Light theme |
| Frappe | Mid-light dark theme |
| Macchiato | Mid-dark theme |
| Mocha | Darkest theme |

Each flavor defines 26 named colors: 14 accent colors and 12 base/surface
colors. The theme system maps these named colors to Mosaico's UI elements
with sensible defaults that can be overridden per-element.

## Catppuccin Palette

Each flavor provides these named colors:

**Accent colors (14):**
Rosewater, Flamingo, Pink, Mauve, Red, Maroon, Peach, Yellow, Green, Teal,
Sky, Sapphire, Blue, Lavender

**Base colors (12):**
Text, Subtext1, Subtext0, Overlay2, Overlay1, Overlay0, Surface2, Surface1,
Surface0, Base, Mantle, Crust

Color values are compiled into the binary as constants -- no external files
or crate dependencies are needed. The hex values for all four flavors are
sourced from the official Catppuccin specification.

## Architecture

### New Files

```
crates/
  mosaico-core/
    src/
      theme/
        mod.rs              # Theme, Flavor, CatppuccinColor enums, resolve()
        palette.rs          # Hardcoded hex values for all 4 flavors (26 colors each)
        mapping.rs          # ThemeMapping: named color -> UI element defaults
```

### Key Types

- `Flavor` (enum) -- `Latte`, `Frappe`, `Macchiato`, `Mocha`
- `CatppuccinColor` (enum) -- all 26 named colors (`Blue`, `Green`, `Base`,
  `Text`, etc.)
- `ThemeOverrides` (struct) -- optional per-element color overrides using
  `Option<CatppuccinColor>` fields
- `ThemeConfig` (struct) -- `flavor: Flavor`, `overrides: ThemeOverrides`

### Resolved Colors

`resolve(flavor, color) -> String` takes a flavor and a `CatppuccinColor`
and returns the hex string (e.g., `"#89b4fa"` for Mocha Blue). This is the
bridge between the theme system and existing color fields that already
accept hex strings.

## UI Element Mapping

### Default Color Assignments

| UI Element | Config Field | Default Color | Rationale |
|------------|-------------|---------------|-----------|
| Focused border | `borders.focused` | `Blue` | Primary accent |
| Monocle border | `borders.monocle` | `Green` | Distinct from focused |
| Bar background | `bar.colors.background` | `Base` | Main background surface |
| Bar foreground | `bar.colors.foreground` | `Text` | Primary text |
| Bar active workspace | `bar.colors.active_workspace` | `Blue` | Matches focused border |
| Bar inactive workspace | `bar.colors.inactive_workspace` | `Surface1` | Subtle, recessed |
| Bar separator | `bar.colors.separator` | `Surface0` | Between background and surface |
| Bar accent | `bar.colors.accent` | `Mauve` | Alerts, notifications |

Every default can be overridden in the theme config by specifying a different
`CatppuccinColor` name.

## Configuration

New `[theme]` section in `config.toml`:

```toml
[theme]
flavor = "mocha"    # latte, frappe, macchiato, mocha
```

With optional per-element overrides:

```toml
[theme]
flavor = "mocha"

[theme.overrides]
focused_border = "Lavender"       # Override: use Lavender instead of Blue
monocle_border = "Teal"           # Override: use Teal instead of Green
bar_active_workspace = "Mauve"    # Override: use Mauve instead of Blue
```

### Theme Disabled

If no `[theme]` section is present, the existing manual hex color
configuration in `config.toml` and `bar.toml` is used as-is. The theme
system is purely additive -- it does not break existing configurations.

### Theme vs Manual Colors

When a theme is active, it generates hex color values for all mapped UI
elements. These generated values are used **unless** the user has also
specified an explicit hex color in `config.toml` or `bar.toml`. The
precedence is:

1. Explicit hex color in `config.toml` / `bar.toml` (highest)
2. Theme override in `[theme.overrides]`
3. Theme default for the flavor
4. Hardcoded application default (lowest, only if no theme)

## Integration

### Application Flow

1. `load()` reads `config.toml` including the `[theme]` section
2. If a theme is configured, `resolve_theme(config)` generates hex colors
   for all mapped UI elements
3. Generated colors fill in any `config.toml` / `bar.toml` fields that the
   user has not explicitly set
4. `TilingManager`, `Bar`, and `Border` receive the final resolved colors
   as they do today -- they are unaware of the theme system

### Hot-Reload

Theme changes in `config.toml` (changing flavor or overrides) trigger the
existing `ConfigReload::Config` path. The daemon resolves the new theme
colors and calls `reload_config()` / updates the bar as usual.

### Doctor Integration

`mosaico doctor` validates:
- The flavor name is one of the four recognized values
- Override color names are valid `CatppuccinColor` variants

### Init Template

`mosaico init` generates a commented `[theme]` section in `config.toml`
showing the available flavors and override fields. The section is commented
out by default so existing behavior is preserved.

## Palette Data

The hex values for all four flavors are hardcoded as `const` arrays or
match arms in `palette.rs`. Example for Mocha:

```rust
// Mocha accent colors
Rosewater => "#f5e0dc",
Flamingo  => "#f2cdcd",
Pink      => "#f5c2e7",
Mauve     => "#cba6f7",
Red       => "#f38ba8",
Maroon    => "#eba0ac",
Peach     => "#fab387",
Yellow    => "#f9e2af",
Green     => "#a6e3a1",
Teal      => "#94e2d5",
Sky       => "#89dcfe",
Sapphire  => "#74c7ec",
Blue      => "#89b4fa",
Lavender  => "#b4befe",
// Mocha base colors
Text      => "#cdd6f4",
Subtext1  => "#bac2de",
Subtext0  => "#a6adc8",
Overlay2  => "#9399b2",
Overlay1  => "#7f849c",
Overlay0  => "#6c7086",
Surface2  => "#585b70",
Surface1  => "#45475a",
Surface0  => "#313244",
Base      => "#1e1e2e",
Mantle    => "#181825",
Crust     => "#11111b",
```

All four flavor palettes (104 colors total) are included.

## Tasks

- [ ] Create `theme/` module in `mosaico-core` with `Flavor` and `CatppuccinColor` enums
- [ ] Hardcode all four Catppuccin flavor palettes in `palette.rs`
- [ ] Implement `resolve(flavor, color) -> String` for hex lookup
- [ ] Define `ThemeOverrides` struct with `Option<CatppuccinColor>` fields
- [ ] Define `ThemeConfig` struct with serde support
- [ ] Add `[theme]` section to `Config` with `Option<ThemeConfig>`
- [ ] Implement `resolve_theme()` to generate hex colors from theme config
- [ ] Integrate theme resolution into config loading pipeline
- [ ] Ensure explicit hex colors in `config.toml` / `bar.toml` take precedence
- [ ] Update `mosaico init` template with commented `[theme]` section
- [ ] Add theme validation to `mosaico doctor`
- [ ] Verify hot-reload works with theme flavor/override changes
- [ ] Build, lint, test
- [ ] Update documentation

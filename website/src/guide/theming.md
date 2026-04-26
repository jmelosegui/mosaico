# Theming

Mosaico ships with three built-in theme families: [Catppuccin](https://catppuccin.com/),
[Rosé Pine](https://rosepinetheme.com/), and [Tokyo Night](https://github.com/folke/tokyonight.nvim).
Each family has multiple flavors that share a common palette and only differ
in shading, so the focus borders, monocle border, unfocused borders, and the
status bar all stay coordinated.

## Configuration

Set the theme name and flavor in `config.toml`:

```toml
[theme]
name = "catppuccin"   # catppuccin, rose-pine, tokyo-night
flavor = "mocha"
```

## Available Themes

### Catppuccin

| Flavor | Style |
|--------|-------|
| `latte` | Light |
| `frappe` | Medium-dark |
| `macchiato` | Dark |
| `mocha` | Darkest (default) |

### Rosé Pine (`name = "rose-pine"`)

| Flavor | Style |
|--------|-------|
| `main` | Dark, default |
| `moon` | Slightly lighter dark |
| `dawn` | Light |

### Tokyo Night (`name = "tokyo-night"`)

| Flavor | Style |
|--------|-------|
| `night` | Dark, default |
| `storm` | Dark with bluish tint |
| `day` | Light |

## Named Colors

When a theme is active, you can use named colors anywhere a color value is
expected (borders, bar colors). Each theme family exposes its own palette,
plus a small set of common aliases (`red`, `green`, `blue`, `yellow`) that
resolve under every theme so portable configs like `focused = "blue"` look
right regardless of which theme you pick.

### Catppuccin (14 accent colors)

| Name | Description |
|------|-------------|
| `rosewater` | Warm pink |
| `flamingo` | Soft pink |
| `pink` | Pink |
| `mauve` | Purple |
| `red` | Red |
| `maroon` | Dark red |
| `peach` | Orange |
| `yellow` | Yellow |
| `green` | Green |
| `teal` | Teal |
| `sky` | Light blue |
| `sapphire` | Blue |
| `blue` | Vivid blue |
| `lavender` | Soft purple |

### Rosé Pine

| Name | Description |
|------|-------------|
| `love` | Red/pink accent (alias `red`) |
| `gold` | Warm yellow (alias `yellow`) |
| `rose` | Soft pink |
| `pine` | Teal/blue accent (alias `blue`) |
| `foam` | Cyan (alias `green`, `teal`) |
| `iris` | Purple (alias `mauve`, `purple`, `magenta`) |

### Tokyo Night

| Name | Description |
|------|-------------|
| `red` | Red |
| `orange` | Orange |
| `yellow` | Yellow |
| `green` | Green |
| `cyan` | Cyan |
| `teal` | Teal |
| `blue` | Blue |
| `purple` | Purple (alias `mauve`, `magenta`) |

Additional named colors for surfaces and text:

| Name | Description |
|------|-------------|
| `base` | Base background |
| `mantle` | Darker background |
| `crust` | Darkest background |
| `surface0` | Surface layer 0 |
| `surface1` | Surface layer 1 |
| `surface2` | Surface layer 2 |
| `overlay0` | Overlay layer 0 |
| `overlay1` | Overlay layer 1 |
| `overlay2` | Overlay layer 2 |
| `subtext0` | Subdued text |
| `subtext1` | Slightly subdued text |
| `text` | Primary text color |

## Color Resolution

Colors are resolved in the following order:

1. **Hex value** (`"#1e1e2e"`) -- used as-is
2. **Named color** (`"blue"`) -- looked up in the active theme palette
3. **Empty string** -- falls back to the default for that setting

Existing hex color values in your configuration continue to work unchanged
after enabling a theme. Named colors are optional.

## Examples

Using named colors in `bar.toml`:

```toml
[colors]
background = "base"
foreground = "text"
accent = "blue"
```

Using named colors for borders in `config.toml`:

```toml
[borders.colors]
focused = "blue"
monocle = "green"
```

## Hot-Reload

Theme changes in `config.toml` are hot-reloaded. When the flavor changes,
all named colors are re-resolved against the new palette and the bar is
updated without restarting.

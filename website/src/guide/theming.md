# Theming

Mosaico includes a built-in theme system based on the
[Catppuccin](https://catppuccin.com/) color palette. Themes provide a
consistent color scheme across focus borders, monocle borders, and the
status bar.

## Configuration

Set the theme flavor in `config.toml`:

```toml
[theme]
flavor = "mocha"   # latte, frappe, macchiato, mocha
```

## Available Flavors

| Flavor | Style |
|--------|-------|
| `latte` | Light theme |
| `frappe` | Medium-dark theme |
| `macchiato` | Dark theme |
| `mocha` | Darkest theme (default) |

## Named Colors

When a theme is active, you can use named colors anywhere a color value is
expected (borders, bar colors). The following 14 accent colors are available:

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
[borders]
focused = "blue"
monocle = "green"
```

## Hot-Reload

Theme changes in `config.toml` are hot-reloaded. When the flavor changes,
all named colors are re-resolved against the new palette and the bar is
updated without restarting.

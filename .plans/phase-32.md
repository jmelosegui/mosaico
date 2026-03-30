# Phase 32: Rosé Pine and Tokyo Night Themes

**Status:** Pending

**Goal:** Add two new built-in color themes alongside the existing Catppuccin
theme: Rosé Pine (3 flavors) and Tokyo Night (3 flavors).

## Overview

Mosaico currently ships only with Catppuccin (mocha, macchiato, frappé,
latte). This phase adds six new theme variants across two popular theme
families, giving users more choices without external configuration.

## New Themes

### Rosé Pine

| Flavor | Type | Base | Accent (Pine) |
|--------|------|------|----------------|
| `main` | Dark | `#191724` | `#31748f` |
| `moon` | Dark | `#232136` | `#3e8fb0` |
| `dawn` | Light | `#faf4ed` | `#286983` |

Named colors: love, gold, rose, pine, foam, iris (6 accent colors)

### Tokyo Night

| Flavor | Type | Base | Accent (Blue) |
|--------|------|------|---------------|
| `night` | Dark | `#1a1b26` | `#7aa2f7` |
| `storm` | Dark | `#24283b` | `#7aa2f7` |
| `day` | Light | `#d5d6db` | `#34548a` |

Named colors: red, orange, yellow, green, cyan, blue, purple (7 colors)

## Configuration

```toml
# Rosé Pine
[theme]
name = "rose-pine"
flavor = "main"       # main, moon, or dawn

# Tokyo Night
[theme]
name = "tokyo-night"
flavor = "night"      # night, storm, or day
```

## Architecture

The existing theme system is designed for this — `ThemeConfig` already
has `name` and `flavor` fields, and `theme.rs` comments mention
`name = "tokyo"` as a future example. The work is:

1. **Extend `Theme` enum** with 6 new variants:
   - `RosePineMain`, `RosePineMoon`, `RosePineDawn`
   - `TokyoNight`, `TokyoStorm`, `TokyoDay`

2. **Add palettes** — new palette modules or sections in `palette.rs`
   with named color tables and bar color mappings for each variant.

3. **Extend `ThemeConfig::resolve()`** to match "rose-pine" and
   "tokyo-night" theme names.

4. **Extend `Theme` methods** — `border_focused()`, `border_monocle()`,
   `named_color()`, and `bar_colors()` for the new variants.

5. **Update config template** — add commented examples for the new themes.

## Named Color Mapping

Rosé Pine and Tokyo Night use different accent names than Catppuccin.
To keep `named_color()` consistent, map them to a shared set plus
theme-specific names:

### Rosé Pine named colors
| Name | Role |
|------|------|
| love | Red/pink accent |
| gold | Yellow/warm accent |
| rose | Soft pink accent |
| pine | Teal/blue accent |
| foam | Cyan accent |
| iris | Purple accent |
| red | Alias for love |
| yellow | Alias for gold |
| blue | Alias for pine |
| green | Alias for foam |

### Tokyo Night named colors
| Name | Role |
|------|------|
| red | `#f7768e` |
| orange | `#ff9e64` |
| yellow | `#e0af68` |
| green | `#9ece6a` |
| cyan | `#73daca` |
| blue | `#7aa2f7` |
| purple | `#bb9af7` |
| magenta | Alias for purple |
| mauve | Alias for purple |

Common color names (red, blue, green, yellow) should resolve correctly
regardless of theme, so users can write `focused = "blue"` and get
the right shade.

## Border Colors

| Theme | Focused | Monocle |
|-------|---------|---------|
| Rosé Pine Main | Pine `#31748f` | Foam `#9ccfd8` |
| Rosé Pine Moon | Pine `#3e8fb0` | Foam `#9ccfd8` |
| Rosé Pine Dawn | Pine `#286983` | Foam `#56949f` |
| Tokyo Night | Blue `#7aa2f7` | Green `#9ece6a` |
| Tokyo Storm | Blue `#7aa2f7` | Green `#9ece6a` |
| Tokyo Day | Blue `#34548a` | Green `#33635c` |

## Modified Files

```
crates/mosaico-core/src/config/
  theme.rs            # Extend Theme enum, resolve(), border methods
  palette.rs          # Add rose-pine and tokyo-night color tables
                      # (or split into palette/ module if too large)
  template_config.rs  # Add commented theme examples
```

## Tasks

- [ ] Add `RosePineMain`, `RosePineMoon`, `RosePineDawn` to `Theme` enum
- [ ] Add `TokyoNight`, `TokyoStorm`, `TokyoDay` to `Theme` enum
- [ ] Add Rosé Pine named color tables (main, moon, dawn)
- [ ] Add Tokyo Night named color tables (night, storm, day)
- [ ] Add bar color mappings for all 6 new variants
- [ ] Extend `ThemeConfig::resolve()` for "rose-pine" and "tokyo-night"
- [ ] Extend `border_focused()` and `border_monocle()` for new variants
- [ ] Add common color aliases (red, blue, green, yellow) for all themes
- [ ] Update config template with commented examples
- [ ] Split `palette.rs` into module if file exceeds 150 lines
- [ ] Add unit tests for each new variant
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets -- -D warnings`
- [ ] Run `cargo test --lib`
- [ ] Update documentation (website + developer docs)
- [ ] Update `.plans/plan.md`

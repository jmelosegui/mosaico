# Phase 20: Rounded Corners

**Status:** Complete

**Goal:** Add a `corner_style` configuration property to `[borders]`
that rounds both the border overlay and the tiled windows themselves.

## Overview

The border overlay is drawn via per-pixel rasterization into a 32-bit
BGRA DIB. This phase replaces the sharp rectangular edge test with a
rounded-rect frame test. Additionally, `DwmSetWindowAttribute` with
`DWMWA_WINDOW_CORNER_PREFERENCE` is used to apply rounded corners to
tiled application windows on Windows 11+.

## Configuration

### `config.toml`

```toml
[borders]
width = 4
corner_style = "small"   # "square", "small", or "round"
```

| Value | Border overlay | DWM (Windows 11) |
|-------|---------------|------------------|
| `"square"` | Sharp corners (0 px) | `DONOTROUND` |
| `"small"` | Subtle rounding (8 px) | `ROUNDSMALL` (~4 px) |
| `"round"` | Standard rounding (16 px) | `ROUND` (~8 px) |

## Architecture

### Border Overlay

The pixel loop in `border.rs` uses a rounded-rect hit test:

- Outer shape: `in_rounded_rect(px, py, w, h, radius)`
- Inner cutout: `in_rounded_rect(px - bw, py - bw, iw, ih, inner_r)`
- A pixel is on the border if it is inside the outer shape but
  outside the inner cutout

### Window Corners (DWM)

`DwmSetWindowAttribute` with `DWMWA_WINDOW_CORNER_PREFERENCE`:

| Style | DWM value |
|-------|-----------|
| `Square` | `DWMWCP_DONOTROUND` (sharp corners) |
| `Small` | `DWMWCP_ROUNDSMALL` (~4 px) |
| `Round` | `DWMWCP_ROUND` (~8 px) |

The DWM API only supports these 3 presets. The `CornerStyle` enum maps
directly to the presets, avoiding confusing numeric values. The border
overlay uses `CornerStyle::border_radius()` to get the pixel value.
Fails silently on Windows 10 where the attribute does not exist.
Corners are reset to `DWMWCP_DEFAULT` when a window is removed from
management.

## Tasks

- [x] Add `CornerStyle` enum (`Square`, `Small`, `Round`) to config
- [x] Add `corner_style: CornerStyle` to `BorderConfig` (default `Small`)
- [x] Add `corner_style = "small"` to `generate_config()` template
- [x] Add `in_rounded_rect()` helper to `border.rs`
- [x] Update `Border::show()` and `render()` to accept `radius`
- [x] Replace sharp edge test with rounded-rect frame test
- [x] Add `set_corner_preference(hwnd, CornerStyle)` to `frame.rs`
- [x] Add `reset_corner_preference(hwnd)` to `frame.rs`
- [x] Call `set_corner_preference` when windows are tiled
- [x] Call `reset_corner_preference` when windows are removed
- [x] Apply corner preference on config reload (`apply_corner_preference_all`)
- [x] Pass `corner_style` from `TilingManager` to `border.show()`
- [x] Run `cargo fmt --all`
- [x] Run `cargo clippy --workspace` and fix warnings
- [x] Run `cargo test` and fix failures
- [x] Manual test: verify rounded borders, verify DWM corners,
      verify `"square"` gives sharp corners
- [x] Update documentation (`docs/`, `website/`)
- [x] Update `.plans/plan.md`

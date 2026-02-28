# Phase 19: Media Widget

**Status:** Complete

**Goal:** Add a bar widget that displays the currently playing media
(track title, artist) from system media sources (Spotify, browsers,
etc.) using the Windows media transport controls API.

## Overview

The media widget uses the Windows `GlobalSystemMediaTransportControls`
API (GSMTC) to read the currently playing media session. It displays
the track title and optionally the artist name in a pill on the status
bar, following the same rendering pattern as existing text-based widgets.

## Reference Design

```
[1] 2 3 4 5 6 7 8  [icon]  BSP       [icon Artist - Track Title]  RAM: 24%  02:15:35
                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^
                                       media widget (right-aligned)
```

When nothing is playing, the widget is invisible (0 width), same
pattern as the Update widget.

## Architecture

### Data Flow

1. The bar update cycle (1-second tick) queries GSMTC for the current
   media session
2. The session provides: playback status, title, artist, album
3. The daemon formats "Artist - Title" (or just "Title" if no artist)
   and stores it in `BarState::media_text`
4. The widget renders the text inside a standard pill like other widgets

### Win32 / WinRT API

The media information lives in the WinRT
`Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager`
API:

1. `GlobalSystemMediaTransportControlsSessionManager::RequestAsync()`
   to get the session manager
2. `GetCurrentSession()` to get the active media session
3. `TryGetMediaPropertiesAsync()` to read title, artist, album art
4. `GetPlaybackInfo()` to check if media is actually playing

### Dependency Consideration

The WinRT API requires the `windows` crate features for
`Media_Control` and `Foundation`. These are already available in the
`windows` crate but need new feature flags in `Cargo.toml`. No new
external crates are needed.

### New Files

```
crates/
  mosaico-windows/
    src/
      bar/
        widgets/
          media.rs    # Media session query and widget text
```

### Modified Files

```
crates/
  mosaico-core/
    src/
      config/
        bar.rs        # Add WidgetConfig::Media variant
        template.rs   # Add media to default right widgets
  mosaico-windows/
    src/
      bar/
        widgets/
          mod.rs      # Register media in dispatch + BarState field
    Cargo.toml        # Add Media_Control, Foundation features
```

### Key Types

- `WidgetConfig::Media` -- new enum variant with `enabled`, `icon`,
  and optional `format` field (e.g., `"{artist} - {title}"`)
- `BarState::media_text` -- `String` carrying the formatted media info,
  populated by the daemon on each tick

### Widget Behavior

- **Nothing playing**: widget is invisible (0 width)
- **Playing**: shows "Artist - Title" in a pill with icon
- **Paused**: optionally show with a pause indicator or dim style
- **No media session**: widget is invisible
- Updates every bar refresh cycle (1 second)
- If GSMTC is unavailable (older Windows versions), widget stays hidden

## Configuration

### `bar.toml`

```toml
[[right]]
type = "media"
icon = "\uF001"  # music note
# format = "{artist} - {title}"  # optional custom format
# max_length = 40                # truncate long titles
```

### Widget Configuration

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `true` | Show/hide the widget |
| `icon` | `"\uF001"` | Nerd Font icon |
| `format` | `"{artist} - {title}"` | Display format template |
| `max_length` | `40` | Truncate text beyond this length with "..." |

## Integration Points

- **BarState**: needs a new `media_text: String` field. Empty string means
  nothing is playing (widget hidden). Populated by the daemon on each tick.
- **Widget dispatch**: `widget_text()` in `widgets/mod.rs` returns
  `media::text(state)`. The widget uses the standard pill rendering path
  (no special case needed like Workspaces or ActiveWindow).
- **should_skip()**: media widget is skipped when `media_text` is empty,
  same pattern as the Update widget's `update_text` check.
- **Daemon tick**: the 1-second tick already refreshes CPU/RAM. Media
  state is queried on the same cycle -- no new thread or timer needed.
- **Cargo.toml**: requires `Media_Control` and `Foundation` features in
  the `windows` crate dependency for `mosaico-windows`.

## Design Decisions

- **GSMTC over window scraping**: GSMTC is the standard Windows API
  for media info. It works with Spotify, browsers, VLC, and any app
  that integrates with Windows media transport controls. Scraping
  window titles is fragile and app-specific.
- **Polling on tick, not a dedicated thread**: media state is queried
  on the existing 1-second bar update cycle. The WinRT async call
  completes quickly (~1ms). A dedicated thread adds complexity with
  no benefit at this refresh rate.
- **Text truncation**: long track titles are truncated with "..." to
  prevent the widget from consuming too much bar space. The
  `max_length` field is configurable.
- **Invisible when idle**: same pattern as the Update widget -- no
  visual noise when no media is playing.
- **Format string**: the `{artist}` and `{title}` placeholders allow
  users to customize the display (e.g., show only the title, or
  reverse the order). Simple string replacement, not a full template
  engine.
- **Graceful degradation**: if the WinRT API is unavailable (e.g.,
  Windows Server or older builds), the query returns an empty string
  and the widget stays hidden. No error is shown.

## Tasks

- [x] Add `Media_Control` and `Foundation` WinRT features to
      `mosaico-windows/Cargo.toml`
- [x] Add `WidgetConfig::Media` variant to `bar.rs` with `enabled`,
      `icon`, `format`, and `max_length` fields
- [x] Add `Media` to `WidgetConfig::icon()` and `enabled()` match arms
- [x] Add `media_text: String` to `BarState` struct and `Default` impl
- [x] Create `media.rs` in `bar/widgets/` with:
  - [x] `query_media()` -- GSMTC session query, returns formatted string
  - [x] `text(state)` -- returns `state.media_text` (or truncated version)
- [x] Register `pub mod media` in `widgets/mod.rs`
- [x] Add `WidgetConfig::Media` arm to `widget_text()` dispatch
- [x] Add `should_skip` logic for empty `media_text` (same as Update)
- [x] Populate `media_text` in daemon tick handler via `query_media()`
- [x] Add media widget to `generate_bar()` in `template.rs` (commented
      out by default, since not all users play media)
- [x] Build with `cargo build`
- [x] Run `cargo clippy --workspace` and fix warnings
- [x] Run `cargo fmt --all`
- [x] Run `cargo test` and fix any failures
- [x] Manual test: verify widget appears with Spotify, browser media,
      VLC; verify it hides when nothing is playing; verify truncation
      works; verify graceful behavior when GSMTC is unavailable
- [x] Update documentation (`docs/status-bar.md`)

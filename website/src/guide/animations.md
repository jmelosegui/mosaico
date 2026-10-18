# Window Transitions

Mosaico can move windows to their new place over a short time instead of
having them jump there: windows you swap, and the windows a layout change,
monocle mode, or a window that opens or closes moves. Transitions are on by
default. Windows still snap into place while the Windows "Animation
effects" setting is off, and in a remote desktop session, unless
`follow_system` is turned off.

## Configuration

Transitions are configured in the `[animations]` section of `config.toml`:

```toml
[animations]
enabled = true            # On by default
duration = 180            # How long a transition takes, in milliseconds (0-1000)
easing = "ease-out-cubic" # How windows speed up and slow down on the way
follow_system = true      # Snap while Windows turns animations off,
                          # and in a remote session
```

- **enabled** -- `true` by default; set it to `false` to turn transitions off.
- **duration** -- how long a transition takes, from 0 to 1000 milliseconds.
  Longer values are clamped to 1000, and `0` turns transitions off.
- **easing** -- the curve windows follow:

  | Value | Motion |
  |-------|--------|
  | `"linear"` | The same speed all the way |
  | `"ease-out-quad"` | Starts fast, slows down gently |
  | `"ease-out-cubic"` | Starts fast, slows down into place (the default) |
  | `"ease-in-out-cubic"` | Speeds up, then slows down |
  | `"ease-out-expo"` | The fastest start, then a long, soft landing |
  | `"cubic-bezier(x1, y1, x2, y2)"` | A curve of your own, as CSS writes it |

  All four numbers of a `cubic-bezier` must be between 0 and 1, so windows
  never overshoot their place. Any other value makes `config.toml` invalid:
  a running mosaico keeps the settings it has, and one that starts uses the
  defaults.
- **follow_system** -- when `true` (the default), windows snap into place
  while the Windows setting "Animation effects" is off (Settings,
  Accessibility, Visual effects; "Show animations in Windows" on Windows 10)
  and in a remote desktop session. Set it to `false` to animate anyway.

There is no frame rate setting: windows move on every refresh of the display.

## What Moves and What Snaps

Windows move to their new place when you:

- swap or move windows within a monitor, from the keyboard or by dropping a
  window you drag on another, which slides from where you let go;
- cycle layouts, or toggle monocle mode;
- open or close a window, for the windows that make room or take the space;
- change the gap or the split ratio in `config.toml`.

These snap into place, as before:

- a window that just opened, or that mosaico starts tiling, appears where it
  goes;
- windows a workspace switch shows;
- a window that moves to another monitor;
- windows waiting behind the monocle window;
- everything while monitors change, and when mosaico starts;
- a step that moves more than 16 windows at once;
- minimized and maximized windows, and moves of 4 pixels or less.

## Behavior

- Pressing a key again before the windows arrive continues from where they
  are, without a jump or a restart.
- Grabbing a moving window with the mouse stops it, and the layout follows
  where you drop it, as always.
- Focus borders follow moving windows. A window that changes size on its way
  gets its border back once it is in place.
- A slow application gets fewer frames rather than a backlog, so the desktop
  keeps responding. One whose window has not moved for 50 ms, since its
  transition started or since it took its first frame, at any refresh rate,
  makes its whole step jump: every window moving with it goes straight to its
  place. An application that stops responding while its window moves is left
  where it is until it responds again.
- Windows that are moving when your monitors change stop where they are, and
  go to their place once mosaico has laid out the new monitors.

## Troubleshooting

Run `mosaico doctor`. It shows the transition settings mosaico starts with,
those in `config.toml`, or the defaults when the file is missing or invalid,
whether Windows allows animations right now, and whether windows can be moved
frame by frame, and says why windows snap into place when transitions are
enabled.

- **Nothing animates** -- check that `enabled = true` and that `duration` is
  above 0, and with `follow_system = true`, that "Animation effects" is on and
  you are not in a remote desktop session. With [logging](logging.md) on, a
  line that starts "Transitions are off" says mosaico turned them off for the
  session, as it could not have display frames or this desktop cannot move
  windows frame by frame: windows jump into place until mosaico starts again.
- **One application moves less smoothly** -- applications that are busy, or
  that do not follow the display's scaling (blurry on a scaled display), take
  fewer or coarser steps. They still end up in their place.
- **Windows jump to their places part of the way through** -- one busy
  application makes its whole step jump: when its window has not moved for
  50 ms, since its transition started or since it took its first frame, every
  window moving with it goes straight to its place, so none slides over a
  window that is still in the way.
- **Finding out why** -- with [logging](logging.md) enabled at
  `level = "debug"` when mosaico starts, the log says, while transitions are
  on, why a window that moved snapped into place instead, in one line for a
  whole step that snaps, as when the monitors change or a step moves more
  than 16 windows, and how each transition went: how long it took, how many
  frames each window was sent and how many found it still busy with the one
  before, which window was the slowest, and when each window got to its
  place. With transitions off, it says nothing of placements.

## Hot-Reload

The `[animations]` section is hot-reloaded. Windows already moving keep
going, unless the reload moves them somewhere else, which they then head for
with the new duration, or turns transitions off, which puts them in place at
once.

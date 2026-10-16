# Phase 34: Window Transitions

**Status:** Pending

**Goal:** Animate windows to their new place when they move, swap or resize,
and when the layout changes, without making the daemon slower to respond or
letting the tiler lose track of where windows are.

## Overview

Today every placement is immediate. The tiler emits `Effect::Place`, the
applier calls `WindowControl::place`, and Windows moves the window with one
synchronous `SetWindowPos`. This phase adds an optional transition: the
applier turns a placement into a motion, moves the window through
intermediate rects on every display frame, and ends with the same synchronous
`place` as today, so the end state keeps every guarantee it has now.

Everything that decides lives in core and runs on the injected clock: which
placements animate, the curve, retargeting, and when a motion ends. The
platform only moves windows without waiting for them, and wakes the loop once
per display frame.

The work splits into four steps, one PR each (see Tasks).

## Design Goals

- One frame loop on the daemon thread drives every moving window. All windows
  of a step start together and advance on the same frame, so neighbours stay
  aligned and gaps do not open and close.
- Frames follow the display's refresh, not a fixed frame rate kept with
  sleeps. Progress comes from the clock, so a dropped frame does not slow the
  motion down.
- A new target mid-flight continues from where the window is, at the speed it
  has. A repeated target changes nothing. Pressing a key five times in a row
  gives one smooth path, not five restarts.
- Intermediate frames never wait for an application. A slow or hung
  application gets fewer frames, never a backlog, and never delays hotkeys or
  IPC.
- While a window moves, the motion is the source of truth. Nothing reads its
  position back from the OS mid-flight, and our own frames are recognized as
  echoes: they do not trigger layout and do not count as activity.
- Borders follow without being rendered again on every frame.
- The Windows "Animation effects" setting is honoured.
- With animations off, every effect list and call order stays exactly as it
  is today.

## Configuration

```toml
[animations]
enabled = false          # opt-in until verified on the live desktop
duration = 180           # milliseconds, 0 to 1000
easing = "ease-out-cubic"
# linear, ease-out-quad, ease-out-cubic, ease-in-out-cubic, ease-out-expo,
# or "cubic-bezier(0.2, 0.0, 0.0, 1.0)"
follow_system = true     # no animations while Windows "Animation effects" is off
```

There is no frame rate setting: frames come from the display. The section
hot-reloads like the rest of the config, and a motion in flight finishes with
the settings it started with.

## What Animates

A `Place` animates when all of these hold:

- animations are on, and the system setting allows them;
- the window is visible on an active workspace, and neither minimized nor
  maximized;
- it stays on the same monitor. Cross-monitor moves snap in this phase, since
  core does not know monitor scale factors yet, and an application rescales
  when it crosses into a monitor with another scale;
- it is not a window waiting behind the monocle front, which snaps behind it;
- the step is not a display change, where geometry is in flux and everything
  snaps;
- the step moves at most 16 windows, and this window moves or resizes by more
  than a few pixels.

So swaps, moves within a monitor, ratio changes, layout cycling, monocle on
and off, and the reflow when a window opens or closes all animate. The window
that just opened does not: it is pre-placed at its slot before it shows (see
#28) and appears there.

## Architecture

### Today

```
Tiler::step -> Vec<Effect> -> Applier::apply -> Coalescer -> Applier::run
    -> WindowControl::place          (SetWindowPos, waits for the application)
```

### With transitions

```
Applier::run(Place)   -> Motions::retarget(window, from, to, now)   instead of place
Scheduler frame wake  -> Applier::frame(now)
    -> WindowControl::place_frames(rects in flight)                  never waits
    -> motion done: WindowControl::place + repaint                   exact, as today
```

### Core: the motion model (`mosaico-core/src/motion/`)

- `Easing`: the presets and `cubic-bezier`, solved for x with Newton's method
  and a bisection fallback.
- `Segment`: one rect moving from `from` to `to` over `duration` from `start`.
  The first segment follows the configured easing. A retarget starts a cubic
  Hermite segment from the current rect and velocity to the new target, ending
  at rest, so position and speed stay continuous.
- `Motions`: the table of moving windows. `retarget` is a no-op when the target
  is unchanged, `frame(now)` returns the rects in flight and the motions that
  finished, `end(window)` drops one, and the table says whether frames are
  still needed. The last frame of a motion is exactly its target.
- `policy`: the rules in "What Animates", as a pure function of the effect,
  the plan and the settings.

### Core: the applier

- An animating `Place` becomes a motion. It starts from the window's motion
  rect if the window is already moving, otherwise from the desktop's rect for
  it.
- `Applier::frame(now)` posts the rects in flight in one `place_frames` call. A
  finished motion gets today's `place` and `repaint`, then its border at the
  final rect.
- `Focus` and `PutBehind` apply at once. `CenterCursor` on a moving window uses
  its target rect, not where the window is mid-flight.
- `Hide`, `Minimize`, `Close`, `Untile` and `DropBorder` end the window's motion
  first, the way the coalescer drops held placements today. A `Place` with
  `Timing::Now` retargets.
- The coalescer is unchanged: held moves start their motion when released.

### Core: echoes and the scheduler

- A `LocationChanged` from a window in motion is one of our frames. The echo
  filter drops it, and it does not count as activity for the quiet window.
  Today each such event ends the loop's wait early and pushes held moves to
  `MAX_HOLD`; with 60 frames a second per window that would be constant.
- A new `WindowEvent::MoveStarted` (the user grabbed the window) ends that
  window's motion where it is. The `Moved` at the end of the drag retiles as
  today.
- Frame wakes are not activity either. `Step::Frame` runs after the tasks and
  events of the same wake.

### Borders

`LastDrawn::needs_render` already ignores position, so moving a border is a
restack, not a render. A window that only moves keeps its border on every
frame. A window that resizes hides its border at the start and gets it back,
drawn once, at the final rect: rendering is a CPU loop that takes about 30 ms
for half a 4K screen, too slow for every frame.

### Platform contracts

- `WindowControl::place_frames(&[(WindowId, Rect)]) -> Vec<WindowId>` moves
  windows to intermediate rects without waiting for their applications, and
  never changes their maximized or minimized state. It skips, and returns, the
  windows that are minimized, maximized, not responding, or have not reached
  their previous frame yet. Windows: `SetWindowPos` with `SWP_ASYNCWINDOWPOS`,
  `SWP_NOZORDER` and `SWP_NOACTIVATE`, the frame offset read once per motion,
  `IsIconic`, `IsZoomed` and `IsHungAppWindow`, and `GetWindowRect` compared
  with the last rect posted.
- `FrameClock::want_frames(bool)`: while on, the loop wakes about once per
  display refresh, with the time of the frame. Windows: a pacer thread that
  loops on `DwmFlush` and sends `DaemonMsg::Frame`, with a high-resolution
  waitable timer when composition is unavailable.
- `Desktop::animations_allowed() -> bool`: whether the user allows animations.
  Windows: `SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION)`, read when a
  transition starts.
- `WindowEvent::MoveStarted { window }`: the user started moving or resizing a
  window. Windows: `EVENT_SYSTEM_MOVESIZESTART`.

A window skipped several frames in a row stops getting frames and only gets
its final placement.

## Edge Cases

1. Rapid input: a target that changes mid-flight retargets at the speed the
   window has; the same target again is a no-op.
2. Slow or hung applications: frames never block, and skipped windows catch up
   at the final placement.
3. Minimum sizes: an application may refuse an intermediate size; the final
   `place` sets the exact target, as today.
4. The user grabs a moving window: `MoveStarted` ends its motion.
5. Maximize or minimize mid-flight: `place_frames` skips the window, and the
   motion ends without forcing it back.
6. Close, hide or a workspace switch mid-flight: the motion ends first.
7. Display change mid-flight: every motion ends, and the rebuild snaps.
8. Config reload mid-flight: motions finish with the settings they started
   with.
9. Monocle: turning it on grows the front window to the monocle rect, and
   turning it off shrinks it back to its tile. The other windows do not move.
10. Shutdown: motions end before windows are revealed.

## Later

- Cross-monitor transitions between monitors with the same scale. This needs
  the scale factor in the monitor contract.
- Transitions for a newly opened window and for workspace switches.
- A stand-in mode for heavy resizes: a mosaico-owned window showing a live
  thumbnail of the application scales to the target while the real window is
  placed once. Stand-ins are our own windows, so they can all move in one batch
  per frame. Opt-in, once live resizes have been measured.
- Border rendering whose cost grows with the perimeter, not the area, so
  borders can follow resizes.

## New and Modified Files

```
crates/
  mosaico-core/src/
    motion/                    # NEW: easing, segment, motions table, policy, tests
    tiler/applier.rs           # Place becomes a motion; motions end on Hide, Close, ...
    tiler/applier_motion.rs    # NEW: the frame step
    tiler/echo.rs              # frames from moving windows are echoes
    daemon/scheduler.rs        # frame wakes and frame echoes are not activity
    window.rs                  # place_frames contract
    event.rs                   # MoveStarted
    config/types.rs            # AnimationConfig
    config/mod.rs              # validate: clamp duration
    config/template_config.rs  # [animations] section
    config/store/              # hint for existing configs
    tiler/startup.rs           # Settings
    tiler/lifecycle.rs         # reload
  mosaico-windows/src/
    window/place_frame.rs      # NEW: frames that never wait, one in flight per window
    window_control.rs          # place_frames
    frame_clock.rs             # NEW: the pacer thread
    system_settings.rs         # NEW: the Windows animation setting
    event.rs                   # EVENT_SYSTEM_MOVESIZESTART
    event_source_hooks.rs      # hook range
    daemon_loop_run.rs         # Frame messages
    daemon_threads.rs          # pacer start and stop
docs/
  animations.md                # NEW
  daemon.md                    # DaemonMsg::Frame
```

## Tasks

### 34.1 Motion model and config (core only)

- [ ] `Easing` presets and `cubic-bezier`, with tests for the endpoints,
      monotonic presets and solver accuracy
- [ ] `Segment` with eased and Hermite segments, with tests that the last frame
      is exactly the target and that a retarget keeps position and speed
      continuous
- [ ] `Motions` table (retarget, frame, end), with tests for the no-op
      retarget, dropped frames, and sizes never below 1
- [ ] The policy for which placements animate, with a test for every rule
- [ ] `[animations]` config: types, validation, template, hint for existing
      configs, `Settings` and reload

### 34.2 Platform contracts

- [ ] `WindowControl::place_frames` contract in core and its Windows
      implementation, tested on hidden 1x1 tool windows of the test's own: it
      skips minimized and maximized windows and leaves their state alone
- [ ] `FrameClock` contract, the Windows pacer thread and `DaemonMsg::Frame`
- [ ] `Desktop::animations_allowed` and its Windows implementation
- [ ] `WindowEvent::MoveStarted` from `EVENT_SYSTEM_MOVESIZESTART`
- [ ] Scheduler: frame wakes and frame echoes are not activity, with tests at
      explicit times

### 34.3 Transitions in the applier

- [ ] Animating `Place` effects become motions; `Applier::frame` posts frames
      and finishes with `place` and `repaint`
- [ ] `Focus`, `PutBehind` and `CenterCursor` rules; motions end on `Hide`,
      `Minimize`, `Close`, `Untile`, `DropBorder` and `MoveStarted`
- [ ] The echo filter drops frames from moving windows
- [ ] Borders follow moves, hide during resizes and are drawn once at the end
- [ ] Tiler tests on the fake clock: a swap moves both windows in lockstep and
      ends with exact placements; a second swap mid-flight retargets without a
      jump; a close mid-flight stops its frames; monocle off shrinks the front
      window back to its tile
- [ ] With animations off, every existing effect and call-order test passes
      unchanged

### 34.4 Verification and docs

- [ ] A debug log line per transition: windows, frames posted and skipped,
      and the slowest window
- [ ] `mosaico doctor` shows the animation settings and whether Windows allows
      animations
- [ ] `docs/animations.md`, and `DaemonMsg::Frame` in `docs/daemon.md`
- [ ] Run `cargo fmt --all -- --check`
- [ ] Run `cargo clippy --all-targets -- -D warnings`
- [ ] Run the safe test suites from the testing skill
- [ ] Manual test: swap, move, cycle layouts and toggle monocle with 2 to 6
      windows, including a browser and an Electron app
- [ ] Manual test: a 144 Hz monitor next to a 60 Hz one
- [ ] Manual test: an application that stops responding mid-transition
- [ ] Manual test: Windows "Animation effects" off disables transitions
- [ ] Manual test: CPU use while animating and when idle
- [ ] Decide whether `enabled` defaults to true
- [ ] Update `.plans/plan.md`

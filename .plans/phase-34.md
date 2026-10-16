# Phase 34: Window Transitions

**Status:** Pending

**Goal:** Animate windows to their new place when they move, swap or resize,
and when the layout changes, without making the daemon slower to respond or
letting the tiler lose track of where windows are.

## Overview

Today every placement is immediate. The tiler emits `Effect::Place`, the
applier calls `WindowControl::place`, and Windows moves the window with one
synchronous `SetWindowPos`. This phase adds an optional transition: the
tiler marks the placements that may animate, the applier turns each into a
motion, moves the window through intermediate rects on display frames, and
ends with `place` at the target. Nothing waits for a frame to land: while
one may still be queued, the final `place` and the window's other changes
are posted behind it, and land after it whatever became of it, so the end
state keeps the guarantees it has now once the window gets to it.

Everything that decides lives in core and runs on the injected clock: which
placements animate, the curve, retargeting, when a motion ends and what
happens to a window that falls behind. The platform only moves windows
without waiting for them, reports how each frame went, and wakes the loop on
display frames.

The work splits into four steps, one PR each (see Tasks).

## Design Goals

- One frame loop on the daemon thread drives every moving window. All windows
  of a step start together and advance on the same frames. Each application
  handles its frames when it gets to them, so keeping neighbours aligned is
  best effort: when one window of a step falls behind, every motion of that
  step finishes at its target at once, rather than letting a neighbour slide
  over a window that is still in the way.
- Frames follow the display's refresh, not a fixed frame rate kept with
  sleeps. Progress comes from the clock, so a dropped frame does not slow the
  motion down, and every motion has a deadline, so it finishes even when no
  frame comes.
- A new target mid-flight continues from where the window is, at the speed it
  has, without overshooting. A repeated target changes nothing. Pressing a key
  five times in a row gives one smooth path, not five restarts.
- Intermediate frames never wait for an application. A slow or hung
  application gets fewer frames, never a backlog, and never delays hotkeys or
  IPC. No change is made at once to a window while a frame to it may still be
  on its way: it is posted behind the frame instead, and lands after it.
- While a window moves, its motion is the tiler's record of where it is going.
  The only reads mid-flight are the frame outcomes the platform reports.
- A window's position reports never count as input activity, whoever moved it.
- Borders follow the window, not the frame posted to it, and are never
  rendered again on a frame.
- The Windows "Animation effects" setting is honoured.
- Turning animations off at runtime, by a reload or the system's setting,
  does not undo what was posted: the tiler settles windows while they stay
  off (`FrameControl::settle_responding`), and a window whose application
  responds gets its changes made at once again from `ASYNC_GRACE` after the
  last thing posted to it, while one whose application hangs keeps them
  posted until it responds. Once every window has settled so, as from
  startup, the next goal holds.
- With animations off nothing is posted, so the tiler emits the same effects
  and the platform makes the same calls as today, at once. The one change is
  timing: position reports no longer count as input activity (34.2), so held
  moves are released after the quiet window instead of being pushed to its
  cap while some window reports its position.

## Configuration

```toml
[animations]
enabled = false          # opt-in until verified on the live desktop
duration = 180           # milliseconds, 0 to 1000
easing = "ease-out-cubic"
# linear, ease-out-quad, ease-out-cubic, ease-in-out-cubic, ease-out-expo,
# or "cubic-bezier(0.2, 0.0, 0.0, 1.0)" with both y values from 0 to 1
follow_system = true     # no animations while Windows "Animation effects" is
                         # off, or in a remote session
```

There is no frame rate setting: frames come from the display. The section
hot-reloads like the rest of the config, and a motion in flight finishes with
the settings it started with. The template and the hint added to existing
configs come with 34.4, once something animates.

## What Animates

Two places decide, each with the facts it has.

When the tiler finishes a step (`Plan::finish`), it marks a `Place` as one that
may animate when all of these hold:

- animations are on;
- the window was not shown, adopted or pre-placed in this step. It appears
  where it belongs instead of sliding from where it was hidden or where its
  application put it;
- it is not a window waiting behind the monocle front, which snaps behind it;
- the step is not a display change, the tiler's start, or the bar reservation
  that follows the start, where geometry is in flux and everything snaps;
- it stays on the same monitor. Cross-monitor moves snap in this phase, since
  core does not know monitor scale factors yet, and an application rescales
  when it crosses into a monitor with another scale;
- the step moves at most 16 windows.

When the applier applies a marked `Place`, it animates it when all of these
hold:

- the system allows animations (with `follow_system`);
- the window is neither minimized nor maximized;
- it moves or resizes by more than a few pixels (4);
- the platform supports frames (`place_frames` is not `Unsupported`).

So swaps, moves within a monitor, ratio changes, layout cycling, monocle on and
off, and the reflow when a window opens or closes all animate. The window that
just opened does not, and neither do the windows a workspace switch shows.

## Architecture

### Today

```
Tiler::step -> Vec<Effect> -> Applier::apply -> Coalescer -> Applier::run
    -> WindowControl::place          (SetWindowPos, waits for the application)
```

### With transitions

```
Plan::finish          -> Place { transition: Animate | Snap }       Snap whenever animations are off
Applier::run(Place)   -> Motions::retarget(window, from, to, now)   for Animate, instead of place
Scheduler frame step  -> Applier::frame(now)                        on a frame, or at a motion's deadline
    -> WindowControl::place_frames(frames, outcomes)                never waits
    -> at the deadline: WindowControl::place + repaint              exact, as today, posted behind the frames
```

### Core: the motion model (`mosaico-core/src/motion/`)

- `Easing`: the presets and `cubic-bezier`, solved for x with Newton's method
  and a bisection fallback. A `cubic-bezier` with a y value outside 0 to 1 is
  an invalid value, since it would overshoot.
- `Segment`: one rect moving from `from` to `to` over `duration` from `start`.
  The first segment follows the configured easing. A retarget starts a
  monotone cubic Hermite segment from the current rect and velocity to the new
  target, ending at rest: the start tangent is clamped per component so the
  segment never passes its target, and is 0 when the window was moving away
  from it. Position and speed stay continuous wherever the clamp allows, and
  every frame stays within the box of the start, the rect at the retarget, and
  the target.
- `Motions`: the table of moving windows. `retarget` is a no-op when the target
  is unchanged; `frame(now)` returns the frames to post and the motions that
  are due to finish; the table remembers each window's outcome, counts the
  frames it has been pending, and gives the earliest deadline. The last frame
  of a motion is exactly its target.
- `policy`: the rules in "What Animates", as two pure functions over plain
  facts: the step decision and the apply-time check.

### Core: the tiler marks placements

- `Effect::Place` gains a `transition` field, `Snap` or `Animate`. The layout
  passes emit `Snap`, and `Plan::finish` runs the step decision over the step's
  placements and sets `Animate` where it holds. With animations off nothing is
  marked, so effect lists are as today apart from the field.
- The coalescer keeps the latest placement of each window with its transition,
  and releases it with it.

### Core: the applier

- A `Place` marked `Animate` that passes the apply-time check becomes a motion.
  It starts from the window's motion rect if the window is already moving,
  otherwise from the desktop's rect for it.
- A `Place` marked `Snap` for a moving window finishes that window's motion
  first (below), then places as today.
- The frame step posts every frame due in one `place_frames` call and acts on
  each outcome:
  - handled: the border moves to the rect the window reached;
  - pending: the border stays; a window pending for 3 frames in a row makes
    every motion of its step finish at once;
  - maximized or minimized: the motion stops, and nothing forces the window
    back;
  - gone: the motion is dropped.
- Finishing a motion: at its deadline, today's `place` and `repaint` at the
  target, then its border, drawn once at the final rect. The platform posts
  both behind any frame still queued for the window, so they land after it
  and nothing waits for a frame to land: no finishing state, no waiting, and
  no check after the final placement. A `Snap` placement of a window with
  frames on their way orders itself the same way.
- Stopping a motion: the window stays where it is.
- What ends a motion:
  - `Hide`, `Minimize`, `Destroyed` and `MoveStarted` stop it. A window shown
    or restored later is placed again by the tiler, and that placement snaps.
  - `Close` leaves it running: the application may refuse to close, and its
    `Destroyed` or `Hidden` ends it.
  - `Untile`, `Reveal`, a display change and shutdown finish it at the target.
  - `Destroyed` drops it in `Applier::echo`, with everything held for the
    window.
- `Focus` and `PutBehind` apply at once, and are never held for a motion.
  The platform orders `put_behind` itself, behind what may be queued for the
  window, as it does every change but focus. `Focus { move_cursor: true }` and
  `CenterCursor` on a moving window put the pointer at the center of its
  target, not where the window is mid-flight.
- A `Border` effect for a moving window only replaces the style the motion
  will draw (a color, a width, or hidden); the frame step positions the border.

### Core: the scheduler

- `LocationChanged` events and frames never count as activity, for any window:
  a wake that brought only those counts like a timeout for flushing. Position
  reports still reach the tiler, which needs them to notice a focused window
  being maximized.
- `begin_wait` waits no longer than the earliest motion deadline, and reaching
  it runs the frame step.
- All frames of a wake fold into one `Step::Frame`, after the wake's tasks and
  events. The frame step takes its time from `Platform::now()`.
- `MouseHover` is dropped while any motion is in flight, so a window sliding
  under a still pointer does not take focus.

### Borders

`LastDrawn::needs_render` already ignores position, so moving a border is not a
render. During a motion a border moves to the rect its window reached, with a
position-only move, and stays put while the window is pending. A window that
resizes hides its border at the start and gets it back, drawn once, at the
final rect: rendering is a CPU loop that takes about 30 ms for half a 4K
screen, too slow for every frame.

### Platform contracts

- `FrameControl::place_frames(&self, frames: &[Frame], outcomes: &mut
  Vec<FrameOutcome>) -> Result<()>`, which every `WindowControl` is, where a
  `Frame` is a window, a rect and whether it is the first frame of a motion
  (the platform then reads the window again). It moves windows to
  intermediate rects without waiting for their applications, and never
  changes their maximized or minimized state. The decisions are core's
  `FrameTable`, which reads windows through a `FrameProbe`. A frame posted
  less than `PENDING_LIMIT` (250 ms) ago to a window that has not been seen to
  move since holds the next one back, so a slow application gets at most one
  frame that often; this is a backlog limit only, and nothing depends on
  knowing whether a frame landed. The outcome, per frame: `Handled { reached }`
  (the frame was posted, or was not needed because it maps to where the
  window is, or the system refused the post, with the window's visible rect
  before it lands), `Pending` (nothing posted: the last frame is recent and the
  window has not been seen to move since, or the application does not
  respond), `Maximized`, `Minimized`, `Gone`. A window whose owner changed
  (a reused id) starts over with a first frame and is `Gone` for any other. A
  frame for a window that does not follow the display's scale is mapped to its
  own pixels first. A platform without frames returns `Error::Unsupported`,
  and the tiler then places windows without transitions. Windows:
  `SetWindowPos` with `SWP_ASYNCWINDOWPOS`, `SWP_NOZORDER`, `SWP_NOACTIVATE`
  and `SWP_NOSENDCHANGING` (as `set_rect`, so the minimum size an application
  asks for clamps no frame that the final place would not), and never
  `SWP_NOCOPYBITS` (it would force a full repaint on every frame of a plain
  move), nor `SWP_FRAMECHANGED` on resize frames; the frame offset read once
  per motion; `IsIconic`, `IsZoomed` and `IsHungAppWindow`.
- A window's other changes keep their order with its frames: `place`,
  `repaint`, `put_behind`, `minimize`, `hide`, `show` and `reveal` are posted
  behind what may be queued for the window while `FrameTable::posts_behind`
  holds: less than `ASYNC_GRACE` (5 s) after anything was posted to it; after
  that, for as long as it has not been seen to move since the last thing
  posted to it, a frame or any other change; and, once it was given a frame,
  while its application does not respond. Past the grace, any move of the
  window, whoever made it, ends the order. A window never given a frame is
  changed at once, as today, responding or not, and so is one
  `FrameControl::settle_responding` forgot: the tiler calls it while
  transitions are off, and it forgets every window something was last posted
  to more than `ASYNC_GRACE` before, whose application responds.
  `FrameControl::posts_behind(window)` tells the caller which it is: a posted
  change may complete after the call returns, and the desktop may not show it
  until then, so the caller never reads it back as done. A posted `place`
  ends a maximized state too, that of a window minimized when it is posted
  which a show queued ahead brings back maximized included, and lands with
  the window's invisible borders in its normal state, at its display's scale
  now. Focus is never posted. Callers never wait for a frame before placing,
  hiding or showing a window. Windows: `SetWindowPos` with
  `SWP_ASYNCWINDOWPOS` and the flags of `set_rect`, after
  `ShowWindowAsync(SW_SHOWNOACTIVATE)` for a maximized window in view or a
  minimized one a show queued ahead brings back in view (one out of view has
  `WS_MAXIMIZE` cleared at once, which waits for its application, once it is
  hidden and unless the application does not respond: else it stays
  maximized, and a later show restores its old normal bounds), `RedrawWindow`
  without `RDW_UPDATENOW`, `ShowWindowAsync`, and for `put_behind`
  `SWP_ASYNCWINDOWPOS` too.
  Cloaking does not go through the window's queue, and is unaffected.
- `FrameClock::want_frames(bool)`: while on, the loop is woken about once per
  display refresh. One long-lived pacer, parked while frames are not wanted;
  `want_frames` only sets a flag and never blocks; at most one frame message
  waits in the queue. Windows: a thread that waits on `DwmFlush`, falling back
  to a high-resolution waitable timer at about 60 Hz when `DwmFlush` fails or
  returns within a few ms (it can return at once while the display sleeps or
  the session is locked). The loop is written against an injected wait so it is
  tested without DWM, and stopping the daemon never waits on `DwmFlush` longer
  than a frame.
- `Desktop::animations_allowed() -> bool`: whether the user allows animations,
  read when a transition starts. Windows:
  `SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION)`, the setting behind both
  the Windows 11 "Animation effects" and the Windows 10 "Show animations in
  Windows" toggles, and false in a remote session.
- `WindowEvent::MoveStarted { window }`: the user started moving or resizing a
  window. Windows: `EVENT_SYSTEM_MOVESIZESTART`, already inside the system hook
  range.

## Edge Cases

1. Rapid input: a target that changes mid-flight retargets from the window's
   rect and speed without overshooting; the same target again is a no-op.
2. Slow or hung applications: frames never block, and a slow application gets
   at most one frame per `PENDING_LIMIT`. A window pending for 3 frames makes
   its step finish at once, and its final placement is posted behind its
   frames, so it lands after them.
3. Minimum sizes: frames and the final `place` skip the application's minimum
   size alike, so nothing stops short and jumps at the end.
4. The user grabs a moving window: `MoveStarted` stops its motion.
5. Maximize or minimize mid-flight: the outcome says so, and the motion stops
   without forcing the window back.
6. Close mid-flight: the motion runs on until the window is destroyed or
   hidden, so a refused close still ends in the slot.
7. Hide, minimize or a workspace switch mid-flight: the motion stops, and the
   hide or minimize is posted behind the frames on their way, landing after
   them. The window is placed again, snapping, when it comes back.
8. Display change mid-flight: every motion finishes at its target, and the
   rebuild snaps.
9. Config reload mid-flight: motions finish with the settings they started
   with.
10. Monocle: turning it on grows the front window to the monocle rect, and
    turning it off shrinks it back to its tile. The other windows do not move.
11. Focus follows mouse: hover reports are dropped while windows move.
12. Mouse follows focus: the pointer goes to the center of the focused window's
    target.
13. Shutdown: motions finish at their targets before windows are revealed.

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
    tiler/effect.rs            # Place { transition }
    tiler/plan.rs              # the step decision in finish
    tiler/coalesce.rs          # keeps the transition of held placements
    tiler/applier.rs           # Place becomes a motion; what ends one
    tiler/applier_motion.rs    # NEW: the frame step and finishing motions
    tiler/window_effects.rs    # cursor paths use a moving window's target
    tiler/manager.rs           # frames, deadlines, settings
    tiler/manager_inputs.rs    # reloads reach the applier
    daemon/scheduler.rs        # position reports and frames are not activity; deadlines
    daemon/frame_clock.rs      # NEW: FrameClock
    frames.rs, frames/         # FrameControl, Frame, FrameOutcome, FrameTable, FrameProbe
    tiler/desktop.rs           # animations_allowed
    event.rs, event/naming.rs  # MoveStarted
    config/types.rs            # AnimationConfig
    config/mod.rs              # validate: clamp duration
    config/template_config.rs  # [animations] section (34.4)
    config/store/              # hint for existing configs (34.4)
    tiler/startup.rs           # Settings
    tiler/lifecycle.rs         # reload
    tiler/tests/               # fakes: frames that land late, outcomes, metrics scenario
  mosaico-windows/src/
    window_control/frames.rs   # NEW: frames that never wait, one per limit to a window that has not moved
    window_control.rs          # place_frames; placements posted behind frames
    window_control/hiding.rs   # show state changes posted behind what was posted
    frame_clock.rs             # NEW: the pacer thread
    system_settings.rs         # NEW: the Windows animation setting
    event.rs, event_source.rs  # EVENT_SYSTEM_MOVESIZESTART
    daemon_types.rs            # DaemonMsg::Frame
    daemon_loop_run.rs         # Frame messages, deadlines
    daemon_loop_task.rs        # Frame messages
    daemon_threads.rs          # the pacer
  mosaico/src/commands/test_support/  # fakes implementing the new methods
  mosaico-core/src/bar/tests/         # fakes implementing the new methods
docs/
  animations.md                # NEW
  daemon.md                    # DaemonMsg::Frame, the pacer
  window-management.md         # place_frames
  window-events.md             # MoveStarted
metrics/                       # baselines recorded with each step
```

## Tasks

Each step records its metrics with `cargo xtask metrics --bless` in the commit
that changes them.

### 34.1 Motion model and config (core only)

- [x] `Easing` presets and `cubic-bezier` (y from 0 to 1), with tests for the
      endpoints, monotonic presets and solver accuracy
- [x] `Segment` with eased and monotone Hermite segments, with tests that the
      last frame is exactly the target, that a retarget keeps position and
      speed continuous, and that frames never leave the box of start, retarget
      and target
- [x] `Motions` table, with tests for the no-op retarget, dropped frames and
      sizes never below 1
- [x] The step decision and the apply-time check, with a test for every rule
- [x] `[animations]` config: types, defaults, validation and parsing

### 34.2 Platform contracts

- [x] `FrameControl::place_frames` with `Frame` and `FrameOutcome` in core and
      the Windows implementation, tested on windows of threads that never pump
      and that pump late
- [x] A window's other changes keep their order with its frames: `place`,
      `repaint`, `put_behind`, `minimize`, `hide`, `show` and `reveal` are
      posted behind them while `FrameTable::posts_behind` holds, which
      `FrameControl::posts_behind` tells the caller; focus never is; and
      `FrameControl::settle_responding` lets windows go back to changes made
      at once once transitions are off
- [ ] `FrameClock` contract and the Windows pacer, tested against an injected
      wait; `DaemonMsg::Frame`
- [ ] `Desktop::animations_allowed` and its Windows implementation
- [ ] `WindowEvent::MoveStarted` from `EVENT_SYSTEM_MOVESIZESTART`
- [ ] Scheduler: position reports and frames are not activity, motion
      deadlines, one frame step per wake; tests at explicit times
- [ ] Fakes record frames, can make frames land late, and can say whether
      animations are allowed

### 34.3 Transitions in the tiler and the applier

- [ ] `[animations]` in `Settings` and reload (left out of 34.1, where nothing
      read it yet)
- [ ] `Motions` gives its earliest deadline and takes the frame outcomes of
      34.2
- [ ] `Place { transition }`, set by the step decision in `Plan::finish` and kept
      by the coalescer
- [ ] Marked placements become motions; the frame step posts frames, acts on
      outcomes and finishes motions only once their last frame has landed
- [ ] What ends a motion, effect by effect, and the cursor rules
- [ ] Borders follow the reached rect, hide during resizes and are drawn once at
      the end
- [ ] `MouseHover` dropped while windows move
- [ ] The daemon loop asks for frames while anything moves and runs the frame
      step on frames and deadlines
- [ ] Tiler tests on the fake clock: a swap moves both windows in lockstep and
      ends with exact placements; a second swap mid-flight retargets without a
      jump; frames that land late delay the final place; a window pending for
      3 frames finishes its step; each ending rule; monocle off shrinks the
      front window back to its tile
- [ ] A metrics scenario for a swap with animations on
- [ ] With animations off, the tiler and platform calls of every existing test
      are unchanged

### 34.4 Verification and docs

- [ ] The `[animations]` section in the config template, and the hint for
      existing configs
- [ ] A debug log line per transition: windows, frames posted, pending and
      handled, and the slowest window (no log line per frame)
- [ ] `mosaico doctor` shows the animation settings and whether Windows allows
      animations
- [ ] `docs/animations.md`, and the changes in `docs/daemon.md`,
      `docs/window-management.md` and `docs/window-events.md`
- [ ] Run `cargo fmt --all -- --check`
- [ ] Run `cargo clippy --all-targets -- -D warnings`
- [ ] Run the safe test suites from the testing skill, and `cargo xtask metrics`
- [ ] Manual test: swap, move, cycle layouts and toggle monocle with 2 to 6
      windows, including a browser (resize frames) and an Electron app
- [ ] Manual test: a 144 Hz monitor next to a 60 Hz one
- [ ] Manual test: an application that stops responding mid-transition
- [ ] Manual test: Windows "Animation effects" off disables transitions
- [ ] Manual test: CPU use while animating and when idle
- [ ] Decide whether `enabled` defaults to true
- [ ] Update `.plans/plan.md`

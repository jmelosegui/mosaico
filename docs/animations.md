# Window Transitions

With `[animations] enabled = true`, a window the tiler moves or resizes goes
to its new place over a short time instead of jumping there. That covers
windows that swap or move within a monitor, and the windows that a layout
change, monocle mode, a new split ratio, or a window opening or closing
moves. Other placements snap into place as before, such as those of a window
that just opened and of the windows a workspace switch shows (see
[What Animates and What Snaps](#what-animates-and-what-snaps)).

Transitions are off by default, and stay off by default until they have been
checked on a live desktop: the manual tests in `.plans/phase-34.md` decide
whether `enabled` defaults to `true`. With them off from the start, nothing is
posted to any window, so every change is made at once, and the tiler emits
the effects and the platform gets the calls they always did, but for what
came with transitions and holds either way. A window's report that its
position changed is no longer input that keeps the tiler's held-back moves
waiting, and the start of a drag is input, so it starts that wait over. A
report that a window was minimized is ignored once the window is no longer
minimized. A workspace switch takes a hide reported after it showed the
window again for its own. It takes a restore for its own up to
`ASYNC_GRACE` (5 s) after a show that restores the window, where it used to
take one up to 1.5 s after any show, so a restore the user makes within
1.5 s of a show that restored nothing is now seen.

Everything that decides lives in `mosaico-core` and runs on the time it is
given: which placements animate, the curve, a new target mid-flight, when a
motion ends and what happens to a window that falls behind. The platform only
moves windows without waiting for them, says what it found of each one, keeps
the placement that ends a motion behind the frames posted before it, and
wakes the daemon on display frames.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/config/animations.rs` | `AnimationConfig`, the `[animations]` section |
| `crates/mosaico-core/src/motion/easing.rs`, `bezier.rs`, `easing_names.rs` | `Easing`, the curves, and the names `config.toml` writes them with |
| `crates/mosaico-core/src/motion/segment.rs` | `Segment`: one rect moving from where it is to where it goes |
| `crates/mosaico-core/src/motion/motions.rs` | `Motions`, the table of moving windows: new targets and ends; each frame and what it found in `motions_frame.rs` |
| `crates/mosaico-core/src/motion/policy.rs` | `step_animates` and `still_animates`: which placements animate; `hardly_apart`: how close to a rect a window counts as there |
| `crates/mosaico-core/src/tiler/placement.rs` | `Transition`, which every `Effect::Place` carries |
| `crates/mosaico-core/src/tiler/plan_transitions.rs` | The step decision, as a step finishes |
| `crates/mosaico-core/src/tiler/manager_frames.rs` | `wants_frames()`, `frame_due(after)`, `frame()`, `turn_off_transitions()`: what the daemon drives transitions with |
| `crates/mosaico-core/src/tiler/applier_run.rs` | Each effect as the applier carries it out, the ones that end a motion included |
| `crates/mosaico-core/src/tiler/applier_motion.rs` | Placements that become motions, and the apply-time check |
| `crates/mosaico-core/src/tiler/applier_frames.rs` | The frame step, and the placements that end motions |
| `crates/mosaico-core/src/tiler/applier_ends.rs` | What ends a motion, effect by effect and event by event, and where the pointer goes |
| `crates/mosaico-core/src/tiler/applier_stop.rs` | Ending motions at once: every one as the tiler stops or transitions turn off, and those whose frames failed; and, while transitions are off, having the platform forget the windows it gave frames to |
| `crates/mosaico-core/src/tiler/applier_display.rs` | What a display change does to moving windows: each stops where it is and keeps its target, where it is placed once the change is rebuilt for |
| `crates/mosaico-core/src/tiler/applier_borders.rs` | The borders of moving windows |
| `crates/mosaico-core/src/tiler/applier_posted.rs`, `applier_reports.rs` | What may still be on its way to a window, which the applier does not read back as done, and what a report of the window says of it |
| `crates/mosaico-core/src/tiler/applier_landing.rs` | The windows a posted placement is taking somewhere, until they land: on a report, or at `LANDING_WAIT` |
| `crates/mosaico-core/src/tiler/applier_shows.rs`, `applier_view.rs` | The changes of show state on their way to windows, and the tiler's view of the desktop with each window as the change leaves it |
| `crates/mosaico-core/src/tiler/plan_maximized.rs` | Whether a window placed in a step is still maximized after it, with its changes posted |
| `crates/mosaico-core/src/frames/control.rs`, `frames/` | `FrameControl`: `place_frames`, `posts_behind` and `settle_responding`; `Frame`, `FrameOutcome`, and the `FrameTable` platforms decide with |
| `crates/mosaico-core/src/daemon/scheduler.rs`, `scheduler_frame.rs` | `Scheduler`: when the main loop takes the frame step |
| `crates/mosaico-core/src/daemon/frame_clock.rs`, `frame_pacer.rs`, `frame_wait.rs` | `FrameClock`; `FramePacer`, which wakes the daemon on display frames; and `FrameWait`, what the pacer waits on |
| `crates/mosaico-windows/src/window_control/frames.rs` | Frames on Win32: posted with `SWP_ASYNCWINDOWPOS`, read back with `GetWindowRect` |
| `crates/mosaico-windows/src/window_control/posted_placement.rs` | A placement posted behind what was posted to a window: the restore it may post first, and the borders it moves the window with |
| `crates/mosaico-windows/src/window_control/frames_kept.rs` | What is kept about a window with its frames, and its borders at the DPI of its display now |
| `crates/mosaico-windows/src/frame_clock.rs` | `DisplayWait`: `DwmFlush`, with a timer when it does not wait |
| `crates/mosaico-windows/src/daemon_frames.rs` | Starting the pacer, and a daemon without frames when it cannot start |
| `crates/mosaico-windows/src/system_settings.rs` | Whether Windows allows animations |
| `crates/mosaico/src/commands/doctor_animations.rs` | What `mosaico doctor` says about transitions |

### Key Types

- `AnimationConfig` -- `enabled`, `duration` (milliseconds), `easing`,
  `follow_system`. `is_on()` says whether transitions are on, `spec()` gives
  the `MotionSpec` (duration and easing) a motion starts with
- `Easing` -- `linear`, `ease-out-quad`, `ease-out-cubic`,
  `ease-in-out-cubic`, `ease-out-expo` or `cubic-bezier(x1, y1, x2, y2)`;
  none goes back or past its end, so windows never overshoot
- `Segment` -- a rect moving from `from` to `to` over a duration, eased or,
  after a new target, along a Hermite curve that keeps the speed it can
- `Motions` -- the moving windows, each with its segment, the step it belongs
  to, the rect it was last sent, where a frame last found it, when it was
  last found somewhere new or handled its first frame, and what the last
  frame posted to it
- `Tick` -- what one frame step does: the frames to post (`in_flight`), the
  motions that are over (`finished`), the windows found somewhere new
  (`reached`), and those found maximized, minimized or gone (`stopped`)
- `Transition` -- on `Effect::Place`: `Snap` or `Animate`
- `StepFacts`, `ApplyFacts` -- what the two decisions are made from
- `Frame`, `FrameOutcome` -- a window's rect at a moment of its transition,
  and what `place_frames` found of the window: `Handled { reached }`,
  `Pending`, `Maximized`, `Minimized` or `Gone`
- `FrameClock` -- wakes the daemon about once per display refresh while it
  wants frames

## How a Transition Works

1. **The tiler marks the placements that may animate.** Every `Place` starts
   as `Snap`. As a step finishes, `Plan::finish` runs `step_animates` over
   the step's placements and marks `Animate` where it holds, from facts only
   the step has (see below).
2. **The applier checks again.** As it applies a marked placement, it asks
   `still_animates` with what the desktop says then. When that holds too, the
   window gets a motion in `Motions`, from where its motion has it if it is
   already moving, otherwise from where it was last sent while that may still
   be on its way, as to a busy window, and otherwise from where the desktop
   has it, with the duration and easing configured now. Otherwise the window
   is placed as before. The windows one `apply` or `flush` sets moving form a
   step.
3. **The daemon takes frames.** While `TilingManager::wants_frames()` says
   windows move, the daemon loop asks its `FrameClock` for frames, and the
   `FramePacer` wakes it about once per display refresh (`DaemonMsg::Frame`).
   The scheduler folds the frames of a wake into one frame step, after the
   wake's input, and also runs one when `TilingManager::frame_due(after)`
   comes, the end of the first motion, or of the wait for the first window
   still to land, after the last frame step, so a transition ends on time
   even when no frame comes.
4. **The frame step posts frames.** `TilingManager::frame()` reads the time
   from the platform, and `Motions::frame` works out each window's rect at
   that time along its curve, whatever the time between frames, so a dropped
   frame never slows a motion down. A window whose rect is not the one it was
   last sent gets a frame, and all of them go out in one `place_frames` call,
   which never waits for an application.
5. **The outcomes move it on.** A window that handled the frame before moves
   on, and its border follows it: to where the frame found it, or to the
   frame now posted when the window took the one before exactly. One still
   to handle it stays, and so does its border. One maximized or minimized
   meanwhile stops where it is, and one that is gone is forgotten; what its
   motion kept for its border is dropped.
6. **A motion ends at its end time.** Its window is then placed at its exact
   target with `place` and `repaint`, even when it is there already. Nothing
   waits for a frame to land: the platform posts the placement behind the
   frames still on their way to the window, so it lands after them, whatever
   became of them. Its border is drawn there at once when the placement is
   made at once, or finds the window there, where the last thing sent to it
   took it exactly, as its last frames usually did, or with nothing sent to
   it before, as after a drag. Otherwise the window is still to land
   (`applier_landing.rs`): it is repainted again, and its border drawn
   there, once it reports being there, within 4 pixels on every edge, or
   being anywhere while nothing sent to it may still be on its way; and
   `LANDING_WAIT` (500 ms) after the placement if it has moved since or is
   there by then, as a placement that moves nothing reports nothing. One
   that has not moved by then, as its application has not got to the
   placement, lands on its next report, and the daemon no longer wakes for
   it. Until then the border stays where the window was last found. A new
   motion of the window, a stop on its way, or the user grabbing it ends
   the wait.

A new target mid-flight, as when a key is pressed again before the windows
arrive, continues from where the motion has the window, keeping as much of
its speed as does not carry it past the new target, and comes to rest there
after the full duration, along a curve of its own rather than the configured
easing. A motion that would keep none of its speed, one that has not moved
yet or moves straight away from the new target, eases from there along the
configured easing instead. A target the window already has changes nothing
while its placement still animates, so pressing a key five times gives one
path, not five restarts. A placement that no longer animates, as one the
window has come within 4 pixels of, places the window there at once.

The windows of one step move together, on the same frames, and those that
start at rest along the same curve. When one of them is found still to handle
its frame and has not been seen to move for 50 ms (`MAX_LAG`), since its
motion started or since it handled its first frame, the step has fallen
behind: every motion of that step, and no other, ends at once with its
placement at its target, rather than letting a neighbour slide over a window
that is still in the way. The limit is a time, so it is the same at any
refresh rate. A window that handled its first frame counts from then, so a
first frame step that came late does not make it fall behind; one still to
handle its first frame counts from the start of its motion.

### What Ends a Motion

- `Hide`, `Minimize`, and the window's `Destroyed` and `MoveStarted` events
  stop it where the window is, the tiler's own hide or minimize before it is
  made, since a frame posted after it would land after it. So does a
  `Minimized` or `Hidden` report that is not the tiler's own, when the
  desktop agrees: a hide report from a window still shown, as an application
  recreating its surface sends, and a minimize report from one restored
  again leave it moving. A window shown or restored later is placed again by
  the tiler, and that placement snaps.
- `Close` leaves it running: the application may refuse to close, and its
  `Destroyed` or `Hidden` report ends the motion.
- A placement that snaps, for a window that is moving, ends its motion and
  places the window at once. `Reveal` finishes it at its target. `Untile`
  places the window at its target and draws no border there: the step that
  lets go of it drops its border. The tiler stopping places every moving
  window, and every window a display change stopped, at its target before
  it reveals any, and draws no border there.
- A display change stops every motion where its window is as soon as it is
  noted (`TilingManager::display_changing()`), and each window keeps its
  target (`applier_display.rs`). Until the change is rebuilt for, every
  placement snaps, as the layout it is for is about to be replaced. After the
  rebuild, or when the daemon cannot read the monitors and skips it
  (`TilingManager::display_change_skipped()`), each window stopped that
  nothing placed or took out of view meanwhile is placed at its target, and
  the border its motion kept, or the tiler decided for it meanwhile, is
  drawn there.
- When the platform says it cannot move windows frame by frame, transitions
  are off for the rest of the session, logged once: the windows in flight are
  placed at their targets, and every later placement snaps. So it is when the
  pacer cannot start, for a daemon without display frames. Any other failure
  to post frames is logged once until frames work again, and places the
  windows those frames were for at their targets; transitions stay on.

## What Animates and What Snaps

The tiler lets a placement animate (`motion::step_animates`) when all of these
hold:

- transitions are on: `enabled`, with a `duration` above 0;
- the window does not appear in this step: one shown, taken into the layout
  or placed before it was shown appears where it goes instead of sliding
  there from where it was hidden or where its application put it;
- it does not wait behind the window in monocle, out of sight;
- the step is not a display change, the daemon's start, or the bar
  reservation right after it, where monitors and work areas are in flux;
- the window stays on its monitor. Moving to another monitor snaps, because
  core does not know the scale of each monitor yet, and an application
  rescales itself when it crosses to a monitor with another scale;
- the step moves at most 16 windows (`MAX_ANIMATED_WINDOWS`): every frame
  costs each moving window a repaint.

The applier lets a marked placement animate when no display change is
settling, from when one is noted until it is rebuilt for, and
`motion::still_animates` holds as it applies it, that is when all of these
do:

- the system allows animations, or `follow_system` is off;
- the window is neither minimized nor maximized: frames never change either
  state;
- some edge of the window travels more than 4 pixels (`MIN_TRAVEL`): a nudge
  would repaint the window on every frame without anyone seeing it move;
- the platform can move windows frame by frame.

So swaps, moves within a monitor, ratio changes, cycling layouts, monocle on
and off, and the reflow when a window opens or closes animate. The window that
just opened does not, and neither do the windows a workspace switch shows.

## The Frame Contract

`FrameControl::place_frames(frames, outcomes)`, which every `WindowControl`
has, moves windows to the rects of their frames without waiting for their
applications, and puts one `FrameOutcome` per frame in `outcomes`. In short
(the full contract is in the trait's docs and in
[window-management.md](window-management.md#frames-of-a-transition)):

- A frame posted less than `PENDING_LIMIT` (250 ms) ago to a window that has
  not been seen to move since holds the next one back: `Pending`, and nothing
  posted. A slow application gets at most one frame that often, never a
  backlog. Any move since the post lets the next frame go, wherever the
  window went. A window whose application is not responding is `Pending`
  too, and gets nothing posted.
- `Handled { reached }` means the frame was posted, or was not needed: it
  asks for where the window is, once mapped to what the window can take. A
  post the system refuses is `Handled` too, and holds nothing back.
- It never raises, focuses, shows or hides a window, and never changes
  whether one is maximized or minimized: such a window is reported and left
  alone. A frame's move is reported as the window's position changing, never
  as the user moving it.
- While `FrameControl::posts_behind` holds for a window, its placements,
  `put_behind` and changes of how it is shown, other than cloaking, are
  posted behind what was posted to it rather than made at once, and noted,
  so the next ones go behind them too. A repaint is then made without
  waiting for the application to paint, so it paints after what is queued,
  but it is never noted: it neither waits in the queue nor renews the
  grace. `focus` is never posted. It holds for `ASYNC_GRACE` (5 s) after the
  last frame or other change noted for the window; after that, for as long
  as the window has not been seen to move since, each noted with where the
  window was, so no timer alone ends it; and, once the window was given a
  frame, while its application does not respond. It never holds for a
  window never given a frame. So nothing waits for a frame: the placement
  that ends a motion lands after its frames. A window whose application
  stopped responding before it was given any frame gets every change at
  once, as without transitions, and those calls wait for it as they always
  did. While transitions are off, the tiler has the platform forget the
  windows it gave frames to as soon as each is past the grace with its
  application responding (`FrameControl::settle_responding`), so that
  changes to them are made at once again.
- A posted repaint only repairs what was wrong before what is queued, so the
  window is repainted again once it lands where a posted placement took it.
  A posted placement ends a maximized state. It posts a restore ahead of the
  move for a maximized window to be in view once what is queued lands, and
  for every minimized one a show queued ahead brings back in view. For one
  out of view it clears the maximized style at once, which waits for the
  application, unless that does not respond, or the window is still shown
  with the hide or minimize that takes it out of view still queued: the
  window then stays maximized, and a later show brings it back at the
  normal bounds it had before. A minimized or maximized window lands with
  the borders read with its frames, in its normal state, moved with the
  system's frame widths once its display's DPI has changed, or by the ratio
  for a window the system stretches.
- The applier does not read a posted change back as done
  (`applier_posted.rs`), and asks `posts_behind` right after each change it
  keeps a record of. A placement is skipped only when the desktop has the
  window in place and it was last sent there too. The pointer aims at where
  the window was last sent, and a new motion of the window starts from there.
  A posted placement that finds the window there, where the last thing sent
  to it took it exactly, or with nothing sent to it before, moves nothing
  and needs no second repaint, so the border is drawn at once; the window
  lands from any other, as above, and a `Border` effect for it meanwhile is
  drawn then, around where it was placed. A posted hide, minimize or show
  that changes something has the window as it leaves it in the tiler's view
  of the desktop, one state for each window, until the desktop has it that
  way, the window reports it, or nothing sent to the window may still be on
  its way, and a show stays while a late hide of the window is still
  expected (`applier_shows.rs`). A window placed while it is out of view, or
  is to be, with its changes posted, counts as maximized as long as the
  desktop says so (`plan_maximized.rs`), and a display change shows such a
  window, when hidden rather than cloaked, before it places it.
- A platform without frames returns `Error::Unsupported`, the default; asking
  with no frames, which moves nothing, tells which it is.

Borders follow the window: a frame step moves a window's border with
`Decorations::move_border` to where the platform found the window, or to the
frame now posted when the window took the one before exactly, so it neither
trails a window that keeps up nor runs ahead of a slow one, and leaves it
while the window is still to handle its frame. Drawing a border takes tens of
milliseconds for a large window, too long for a frame, so a window that
changes size on its way has its border hidden until it is in place, and every
border is drawn once, at the end, as the window lands (see
[focus-border.md](focus-border.md#borders-of-moving-windows)). The pointer
goes to the center of a moving window's target, or of where a window was
last sent while that may still be on its way. With focus following the
mouse, a report that the pointer rests on a window is dropped while windows
move, and is not repeated until the pointer moves to another window. A
report takes three polls of the pointer, at least 200 ms, longer than a
default 180 ms transition, so a window that slides under a pointer at rest
is usually reported once it has stopped, and takes focus then. One reported
while windows still move gets focus only once the pointer has moved to
another window and back.

## Configuration

```toml
[animations]
enabled = false           # opt-in until checked on a live desktop
duration = 180            # milliseconds, 0 to 1000
easing = "ease-out-cubic" # linear, ease-out-quad, ease-out-cubic,
                          # ease-in-out-cubic, ease-out-expo, or
                          # "cubic-bezier(x1, y1, x2, y2)", all four from 0 to 1
follow_system = true      # snap while the system turns animations off,
                          # and in a remote session
```

There is no frame rate setting: frames come from the display. A `duration`
outside 0 to 1000 is clamped, and 0 turns transitions off. An easing that is
none of these, or a `cubic-bezier` with a number outside 0 to 1, makes
`config.toml` invalid, as an unknown `corner_style` does. The section
hot-reloads with the rest of `config.toml`. A motion in flight goes on with the
duration and easing it started with while its target stays and its placement
still animates. One whose target the reload's retile moves turns towards it
over the new duration, and a reload that turns transitions off snaps the
windows its retile places, while the platform forgets the windows it gave
frames to as soon as it can, as above.

`mosaico init` writes the section into a new `config.toml`, and the daemon
appends a comment block about it to an existing file that has none (see
[configuration.md](configuration.md#files-a-newer-version-updates)).

## Troubleshooting

`mosaico doctor` shows the settings the daemon starts with, whether the system
allows animations now, and whether windows can move frame by frame, and says
why windows snap when transitions are enabled. Whether the daemon's pacer runs,
and whether it waits on the display or on its timer, is the daemon's own state,
which doctor does not ask the daemon for.

- **Windows never animate.** `mosaico doctor` names the first reason that
  holds. `enabled = false`, or a `duration` of 0, turns transitions off. With
  `follow_system = true`, the default, transitions follow the Windows
  setting "Animation effects" (Settings, Accessibility, Visual effects; on
  Windows 10, "Show animations in Windows"), and are off in a remote
  session, where every frame would travel over the network. The setting is
  read each time a transition starts, so turning it back on applies to the
  next one. Set `follow_system = false` to animate anyway. A desktop that
  cannot move windows frame by frame, or a daemon whose display frames
  cannot be paced, turns them off for the session, which the log says once.
- **A busy or hung application.** Frames never wait for an application: a
  slow one gets its next frame only once it has moved, so its window moves in
  bigger steps, while the daemon, hotkeys and the rest of the desktop keep
  responding. A window that has not been seen to move for 50 ms (`MAX_LAG`),
  since its motion started or since it handled its first frame, at any
  refresh rate, ends its step at once, so its neighbours do not slide over
  it. A window whose application stops responding while it moves gets no
  more frames; its placement, and every other change made to it while the
  application does not respond, is posted behind what was on its way to it,
  and lands once the application responds again, which the window reports,
  and its border is drawn then. Its next motion starts from where it was
  last sent, not from where the desktop still has it. One whose application
  had stopped responding before it was given any frame gets its placement
  made at once, as without transitions, and that call waits for it.
- **An application that does not follow the display's scale.** Windows maps
  every move made to a window that is not per-monitor DPI aware to the
  window's own pixels, so on a scaled display it moves in steps of those
  pixels, and a move smaller than one of them does not move it at all. Frames
  for such windows are mapped the same way before they are posted, and a
  frame that would not move the window is not posted. Its transitions can
  look less smooth than those of other windows, but they end in the same
  place.
- **A window that changes monitors snaps.** That is by design in this
  version, as is a step that moves more than 16 windows.

## Tests

The motion model is tested on its own, against the time it is given
(`motion/tests/`): easings, segments, new targets that keep position and speed
without passing the target, frames, outcomes, windows that fall behind, and
each rule of both decisions. `tiler/tests/` drives the tiling manager with
transitions on, on the fake platform and its fake clock: a swap moving both
windows together and ending with exact placements, a second swap mid-flight,
frames that land late, a window that falls behind or hangs, each way a motion
ends, a display change that keeps the targets, borders drawn once a posted
placement lands, on a report or once the wait for it is over, transitions
turned off while windows are still posted to, the view of posted shows,
hides and minimizes, the pointer and hover. The `move_once_animated`
scenario in `metrics/tiler.toml` counts what a swap with transitions on asks
of the platform. `frames/tests/` checks the outcomes the frame table decides
and what it posts behind frames, and `crates/mosaico-windows` checks them on
hidden windows of threads that pump late or never. `mosaico doctor`'s report
is tested against the fake `World` in
`crates/mosaico/src/commands/tests/doctor_animations_tests.rs` and
`doctor_animations_check_tests.rs`.

`crates/mosaico/tests/window_transitions.rs` checks transitions on the live
desktop, like `window_management.rs`, so it starts a daemon of its own,
launches notepad and runs only on demand, with the desktop suites, never with
the safe ones. It swaps two notepads with `mosaico action move` and samples
both windows every few milliseconds for 1.2 s. Before the swap, it checks
that `debug state` shows the two notepads alone on workspace 8, on one
monitor, at the same y and height and different x, and otherwise fails
saying its setup is wrong, not transitions. With transitions on, taking
400 ms along `linear`, and with `follow_system = false`, each is seen at
several places between its slots, never outside the box of its two slots,
and ends exactly in the other's slot. With transitions off, each is only ever
seen in one slot or the other.

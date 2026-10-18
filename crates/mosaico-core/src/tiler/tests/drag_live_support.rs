//! What the tests of the tiles that follow a drag share: windows on two
//! monitors, the second's in monocle or not, the slots of its halves and
//! quarters, two windows side by side for resizes, and where a border is
//! drawn.

use crate::id::wid;
use crate::tiler::tests::{Harness, MON_W};
use crate::tiler::{Effect, Settings, Timing};
use crate::{Action, Rect, WindowEvent};

/// Where monocle shows its window on the second monitor.
pub(super) const FRONT: Rect = Rect {
    x: MON_W + 8,
    y: 8,
    width: 1904,
    height: 1064,
};

/// The left half of the second monitor.
pub(super) const LEFT: Rect = Rect {
    width: 948,
    ..FRONT
};

/// The right half of the second monitor.
pub(super) const RIGHT: Rect = Rect {
    x: MON_W + 964,
    ..LEFT
};

/// The top of the right half of the second monitor.
pub(super) const TOP_RIGHT: Rect = Rect {
    height: 528,
    ..RIGHT
};

/// The bottom of the right half of the second monitor.
pub(super) const BOTTOM_RIGHT: Rect = Rect {
    y: 544,
    ..TOP_RIGHT
};

/// Windows 1 and 2 on the first monitor, and `ids` on the second, in BSP,
/// with `settings`.
pub(super) fn on_two(settings: Settings, ids: &[u64]) -> Harness {
    let mut windows = vec![(1, 0), (2, 0)];
    windows.extend(ids.iter().map(|&id| (id, 1)));
    Harness::with_settings(settings, 2, &windows)
}

/// As [`on_two`], with the first of `ids` in monocle on the second
/// monitor; then window 1 takes focus back.
pub(super) fn monocle_on_second(settings: Settings, ids: &[u64]) -> Harness {
    let mut h = on_two(settings, ids);
    h.event(WindowEvent::Focused {
        window: wid(ids[0]),
    });
    h.action(Action::ToggleMonocle);
    h.event(WindowEvent::Focused { window: wid(1) });
    h
}

/// Whether `effects` place any window.
pub(super) fn place_any(effects: &[Effect]) -> bool {
    effects.iter().any(|e| matches!(e, Effect::Place { .. }))
}

/// Windows 1 and 2 side by side on one monitor, with `settings`.
pub(super) fn pair(settings: Settings) -> Harness {
    Harness::with_settings(settings, 1, &[(1, 0), (2, 0)])
}

/// Window 1 of a pair, `by` pixels wider.
pub(super) fn widened(by: i32) -> Rect {
    Rect::new(8, 8, 948 + by, 1064)
}

/// Where window 2 of a pair is laid out once window 1 is 200 pixels wider.
pub(super) fn following() -> Rect {
    Rect::new(1164, 8, 748, 1064)
}

/// Window 1, which the user drags, reports that it is at `rect`. Returns
/// the placements that makes.
pub(super) fn report(h: &mut Harness, rect: Rect) -> Vec<(u64, Rect, Timing)> {
    h.window(1).rect = rect;
    h.event(WindowEvent::LocationChanged { window: wid(1) });
    h.placements()
}

/// Where `effects` draw the border of window `id` last, if they draw it.
pub(super) fn border_at(effects: &[Effect], id: u64) -> Option<Rect> {
    effects.iter().rev().find_map(|e| match e {
        Effect::Border { window, style } if *window == wid(id) => style.as_ref().map(|s| s.rect),
        _ => None,
    })
}

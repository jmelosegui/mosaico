//! What the tests of the tiles that follow a drag share: two windows side
//! by side for resizes, and where a border is drawn.

use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{Effect, Settings, Timing};
use crate::{Rect, WindowEvent};

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

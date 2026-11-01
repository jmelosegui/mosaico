//! The tiling manager on a fake platform: a tile taking focus stays
//! behind the floating windows once what was held is applied.

use crate::config::Config;
use crate::id::wid;
use crate::{Action, WindowEvent};

use super::super::fake_platform::{Call, FakePlatform};
use super::super::manager_support::start;

#[test]
fn a_window_opening_while_one_floats_goes_behind_it_after_taking_focus() {
    // Arrange -- window 1 floats in front of tile 2, and window 3 is not
    // open yet.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    platform.edit(3, |w| w.visible = false);
    let mut manager = start(&platform, None, &Config::with_gap(8));
    manager.flush_pending();
    manager.handle_action(&Action::ToggleFloat);
    manager.flush_pending();
    platform.take_calls();

    // Act -- its focus change is held until input goes quiet.
    platform.edit(3, |w| w.visible = true);
    manager.handle_event(&WindowEvent::Created { window: wid(3) });
    manager.flush_pending();

    // Assert -- taking focus brings the tile to the front, so it goes
    // behind the float only afterwards.
    let calls = platform.calls();
    let focused = calls.iter().position(|c| *c == Call::Focus(wid(3)));
    let behind = calls
        .iter()
        .rposition(|c| *c == Call::PutBehind(wid(3), wid(1)));
    assert!(
        matches!((focused, behind), (Some(f), Some(b)) if f < b),
        "{calls:?}"
    );
}

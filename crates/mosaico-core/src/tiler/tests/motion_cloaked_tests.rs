//! Windows cloaked away while what is posted to them goes behind what was
//! posted before: cloaking never waits, so the tiler takes a cloaked window
//! for shown, as the desktop has it, and places it as any shown window.

use crate::config::WorkspaceMode;
use crate::id::wid;
use crate::tiler::tests::displays;
use crate::tiler::tests::manager_support::{start, tiled};
use crate::{Action, WindowEvent};

use super::super::{FakePlatform, Manager, animated, settle, swap};
use super::shown_first;

/// Windows 1 and 2 sit on monitor 1 of two, whose workspaces hide windows
/// by cloaking. They swap, so what is posted to them goes behind what was
/// posted before; window 1 is maximized then, if `maximized`; and monitor
/// 1 switches to workspace 2, which cloaks them.
fn cloaked_away(maximized: bool) -> (FakePlatform, Manager) {
    let mut config = animated();
    config.workspaces.mode = WorkspaceMode::PerMonitor;
    let platform = FakePlatform::new(2, &[(1, 1), (2, 1)]);
    let mut manager = start(&platform, None, &config);
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });
    swap(&mut manager);
    settle(&platform, &mut manager);
    platform.edit(1, |w| w.maximized = maximized);
    manager.handle_action(&Action::GoToWorkspace(2));
    platform.take_calls();
    (platform, manager)
}

#[test]
fn a_window_cloaked_away_after_a_transition_that_a_display_change_brings_back_stays_tiled() {
    // Arrange
    let (platform, mut manager) = cloaked_away(false);

    // Act -- monitor 1 goes away, which brings both back into view, on the
    // primary monitor's workspace in view.
    manager.handle_display_change(displays(1), 0, &[]);

    // Assert -- the tiler took them for shown while cloaked, as the
    // desktop has them, so the rebuild keeps them tiled, and shows them.
    let mut kept = tiled(&manager, 0);
    kept.sort_unstable();
    assert_eq!(kept, [1, 2]);
    for id in [1, 2] {
        let window = platform.window(id);
        assert!(window.visible && !window.cloaked, "{window:?}");
    }
}

#[test]
fn a_maximized_window_cloaked_away_is_placed_before_it_is_shown_as_a_display_change_brings_it_back()
{
    // Arrange -- a cloaked window counts as shown, and a placement posted
    // to it ends its maximized state as it does a shown window's.
    let (platform, mut manager) = cloaked_away(true);

    // Act
    manager.handle_display_change(displays(1), 0, &[]);

    // Assert -- as window 2 is.
    assert_eq!(shown_first(&platform.take_calls(), 1), Some(false));
}

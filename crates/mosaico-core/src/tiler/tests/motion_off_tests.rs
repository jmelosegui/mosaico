//! With animations off nothing is posted, so what the applier keeps for
//! changes on their way leaves the tiler deciding as it does without
//! transitions: the reports of a workspace switch's show are taken for
//! its own, and a window a display change brings into view is placed
//! before it is shown.

use crate::config::{HidingBehaviour, WorkspaceMode};
use crate::id::wid;
use crate::tiler::tests::displays;
use crate::tiler::tests::fake_platform::Call;
use crate::tiler::tests::manager_support::{start, still};
use crate::{Action, WindowEvent};

use super::{FakePlatform, two};

#[test]
fn the_restores_of_a_minimizing_round_trip_are_the_tilers_own_with_animations_off() {
    // Arrange -- workspaces hide windows by minimizing them, and window 1
    // has focus.
    let mut config = still();
    config.layout.hiding = HidingBehaviour::Minimize;
    let (platform, mut manager) = two(&config);
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });

    // Act -- away, with both minimizes reported, and back, with both
    // restores reported.
    manager.handle_action(&Action::GoToWorkspace(2));
    for id in [1, 2] {
        manager.handle_event(&WindowEvent::Minimized { window: wid(id) });
    }
    manager.handle_action(&Action::GoToWorkspace(1));
    manager.flush_pending();
    platform.take_calls();
    for id in [1, 2] {
        manager.handle_event(&WindowEvent::Restored { window: wid(id) });
    }

    // Assert -- nothing follows them.
    let calls = platform.take_calls();
    assert!(calls.is_empty(), "{calls:?}");
}

#[test]
fn a_maximized_window_a_display_change_brings_into_view_is_placed_first_with_animations_off() {
    // Arrange -- windows 1 and 2 sit on monitor 1 of two, whose workspaces
    // hide windows by hiding them. Window 1 is maximized, and monitor 1
    // switches to workspace 2.
    let mut config = still();
    config.layout.hiding = HidingBehaviour::Hide;
    config.workspaces.mode = WorkspaceMode::PerMonitor;
    let platform = FakePlatform::new(2, &[(1, 1), (2, 1)]);
    let mut manager = start(&platform, None, &config);
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });
    platform.edit(1, |w| w.maximized = true);
    manager.handle_action(&Action::GoToWorkspace(2));
    platform.take_calls();

    // Act -- monitor 1 goes away, which brings both into view on the
    // primary monitor.
    manager.handle_display_change(displays(1), 0, &[]);

    // Assert -- placed while hidden, then shown in place.
    let calls = platform.take_calls();
    let placed = calls
        .iter()
        .position(|c| matches!(c, Call::Place(w, _) if *w == wid(1)));
    let shown = calls
        .iter()
        .position(|c| matches!(c, Call::Show(w, _) if *w == wid(1)));
    assert!(placed.is_some() && placed < shown, "{calls:?}");
}

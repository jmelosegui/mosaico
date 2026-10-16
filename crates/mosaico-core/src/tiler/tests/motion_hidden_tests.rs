//! Hides and minimizes posted behind what is on the way to a window: the
//! tiler's view of the desktop has the window as the change leaves it while
//! the change is on its way, so what it does next to the window lands in
//! the right order.

use crate::config::{HidingBehaviour, WorkspaceMode};
use crate::id::wid;
use crate::tiler::tests::displays;
use crate::tiler::tests::fake_platform::Call;
use crate::tiler::tests::manager_support::start;
use crate::{Action, Rect, WindowEvent};

use super::{FakePlatform, Manager, REFRESH, animated, placements, swap};

#[path = "motion_cloaked_tests.rs"]
mod cloaked_tests;

/// Whether window `id` is shown before it is placed among `calls`, if it
/// is both.
fn shown_first(calls: &[Call], id: u64) -> Option<bool> {
    let at = |placed: bool| {
        calls.iter().position(|call| match call {
            Call::Place(w, _) => placed && w.raw() == id,
            Call::Show(w, _) => !placed && w.raw() == id,
            _ => false,
        })
    };
    Some(at(false)? < at(true)?)
}

/// Windows 1 and 2 on monitor 1 of two, whose workspaces hide windows
/// `how`. They swap, and window 1's application is slow, or stops
/// responding once it got to the first frame, and then the window is
/// maximized, if `maximized`. Then monitor 1 switches to workspace 2, and
/// window 1's hide waits behind what was posted to it.
fn hidden_on_its_way(how: HidingBehaviour, slow: bool, maximized: bool) -> (FakePlatform, Manager) {
    let mut config = animated();
    config.layout.hiding = how;
    config.workspaces.mode = WorkspaceMode::PerMonitor;
    let platform = FakePlatform::new(2, &[(1, 1), (2, 1)]);
    let mut manager = start(&platform, None, &config);
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });
    platform.edit(1, |w| w.frame_delay = if slow { 40 } else { 0 });
    swap(&mut manager);
    platform.advance(REFRESH);
    manager.frame();
    platform.edit(1, |w| (w.hung, w.maximized) = (!slow, maximized));
    manager.handle_action(&Action::GoToWorkspace(2));
    platform.take_calls();
    (platform, manager)
}

/// Monitor 1 goes away, which brings window 1 onto the primary monitor's
/// workspace in view; then its application gets to all that is queued for
/// it. Gives where the rebuild placed window 1 last, and whether it showed
/// it before that.
fn brought_back(platform: &FakePlatform, manager: &mut Manager) -> (Option<Rect>, Option<bool>) {
    manager.handle_display_change(displays(1), 0, &[]);
    let calls = platform.take_calls();
    platform.edit(1, |w| w.hung = false);
    platform.catch_up(1);
    let placed = placements(&calls).into_iter().rev().find(|&(w, _)| w == 1);
    (placed.map(|(_, rect)| rect), shown_first(&calls, 1))
}

#[test]
fn a_window_a_display_change_brings_into_view_while_its_minimize_is_on_its_way_is_restored_first() {
    for slow in [true, false] {
        // Arrange
        let (platform, mut manager) = hidden_on_its_way(HidingBehaviour::Minimize, slow, false);

        // Act
        let (placed, _) = brought_back(&platform, &mut manager);

        // Assert -- restored before it is placed, so it ends shown, in the
        // slot the rebuild placed it in.
        let one = platform.window(1);
        assert!(one.visible && !one.minimized, "slow: {slow}, {one:?}");
        assert_eq!(Some(one.rect), placed, "slow: {slow}");
    }
}

#[test]
fn a_window_a_display_change_brings_into_view_while_hidden_is_shown_first_only_if_maximized() {
    for (slow, maximized) in [(true, true), (false, true), (true, false), (false, false)] {
        // Arrange -- a placement posted while it is to be out of view would
        // leave a maximized window maximized, and the show then bring it
        // back where it was before it was maximized.
        let how = HidingBehaviour::Hide;
        let (platform, mut manager) = hidden_on_its_way(how, slow, maximized);

        // Act
        let (placed, first) = brought_back(&platform, &mut manager);

        // Assert -- a maximized one is shown before it is placed, so the
        // placement lands behind the show and ends its maximized state; any
        // other is placed first, to be shown in place. Each ends in its slot.
        let one = platform.window(1);
        let case = format!("slow: {slow}, maximized: {maximized}, {one:?}");
        assert_eq!(first, Some(maximized), "{case}");
        assert!(one.visible && !one.maximized, "{case}");
        assert_eq!(Some(one.rect), placed, "{case}");
    }
}

//! When the platform is asked to forget the windows it can: at the start of
//! a step, before the tiler reads the desktop, and of a flush; and for a
//! window given frames that got none posted, as its application did not
//! respond.

use crate::config::{HidingBehaviour, WorkspaceMode};
use crate::frames::ASYNC_GRACE;
use crate::id::wid;
use crate::tiler::tests::displays;
use crate::tiler::tests::fake_platform::Call;
use crate::tiler::tests::manager_support::{start, still};
use crate::{Action, Direction, WindowEvent};

use super::super::{FakePlatform, animated, settle, swap, two};
use super::{settles, swapped};

#[test]
fn a_step_past_the_grace_with_transitions_off_decides_as_from_startup() {
    // Arrange -- windows 1 and 2 sit on monitor 1 of two, whose workspaces
    // hide windows by hiding them. They swap with transitions on, window 1
    // is maximized, monitor 1 switches to workspace 2, and a reload turns
    // transitions off; then the grace passes.
    let mut config = animated();
    config.layout.hiding = HidingBehaviour::Hide;
    config.workspaces.mode = WorkspaceMode::PerMonitor;
    let platform = FakePlatform::new(2, &[(1, 1), (2, 1)]);
    let mut manager = start(&platform, None, &config);
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });
    swap(&mut manager);
    settle(&platform, &mut manager);
    platform.edit(1, |w| w.maximized = true);
    manager.handle_action(&Action::GoToWorkspace(2));
    for id in [1, 2] {
        platform.catch_up(id);
    }
    config.animations.enabled = false;
    manager.reload_config(&config);
    platform.advance(ASYNC_GRACE * 2);
    platform.take_calls();

    // Act -- monitor 1 goes away, which brings window 1 back into view.
    manager.handle_display_change(displays(1), 0, &[]);

    // Assert -- the windows are forgotten first, so window 1 is placed,
    // then shown, as it is from startup.
    let calls = platform.take_calls();
    assert_eq!(calls.first(), Some(&Call::SettleResponding), "{calls:?}");
    let placed = calls
        .iter()
        .position(|c| matches!(c, Call::Place(w, _) if *w == wid(1)));
    let shown = calls
        .iter()
        .position(|c| matches!(c, Call::Show(w, _) if *w == wid(1)));
    assert!(placed.is_some() && placed < shown, "{calls:?}");
}

#[test]
fn a_flush_past_the_grace_with_transitions_off_makes_its_changes_at_once() {
    // Arrange -- a reload turns transitions off right after a swap, and a
    // move is held. The swap's placements moved nothing, so what is posted
    // after them goes behind them until the windows are forgotten.
    let (platform, mut manager) = swapped();
    let left = platform.window(2).rect;
    manager.reload_config(&still());
    manager.handle_action(&Action::Move(Direction::Left));
    platform.advance(ASYNC_GRACE * 2);

    // Act -- input goes quiet past the grace.
    manager.flush_pending();

    // Assert -- forgotten first, so window 1 is placed at once.
    let one = platform.window(1);
    assert_eq!(one.rect, left, "{one:?}");
    assert!(one.queued.is_empty(), "{one:?}");
}

#[test]
fn a_window_given_frames_while_it_did_not_respond_is_forgotten_once_it_responds() {
    // Arrange -- window 1's application does not respond for a whole swap,
    // so it is given frames but gets none posted. Transitions turn off, and
    // past the grace window 2 is forgotten, but not window 1.
    let (platform, mut manager) = two(&animated());
    platform.edit(1, |w| w.hung = true);
    swap(&mut manager);
    settle(&platform, &mut manager);
    manager.reload_config(&still());
    platform.advance(ASYNC_GRACE * 2);
    manager.handle_action(&Action::Retile);
    platform.take_calls();

    // Act -- its application responds again, and the layout is laid out
    // again.
    platform.edit(1, |w| w.hung = false);
    manager.handle_action(&Action::Retile);

    // Assert -- the platform is asked to forget it too.
    assert_eq!(settles(&platform.take_calls()), 1);
}

//! What a key or a display change leave of a drag the tiler follows, and
//! what ends it.

use crate::id::wid;
use crate::tiler::tests::{Harness, MON_W, displays};
use crate::tiler::{Effect, Input};
use crate::{Action, WindowEvent};

use super::support::{grab, places};

#[test]
fn a_key_pressed_while_a_window_is_dragged_leaves_it_with_the_user() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let slot = h.rect_of(1);
    grab(&mut h, 1);

    // Act -- the user asks for a retile, then drops the window.
    let retiled = h.action(Action::Retile).to_vec();
    let dropped = h.event(WindowEvent::moved(wid(1))).to_vec();

    // Assert -- the retile leaves it where the user has it, and the drop
    // puts it back in its slot.
    assert!(places(&retiled, 2) && !places(&retiled, 1), "{retiled:?}");
    assert!(places(&dropped, 1), "{dropped:?}");
    assert_eq!(h.rect_of(1), slot);
}

#[test]
fn a_display_change_while_a_window_is_dragged_leaves_it_with_the_user() {
    // Arrange -- two monitors, window 2 dragged on the second.
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    grab(&mut h, 2);

    // Act -- the second monitor is disconnected, which takes the window
    // to the first, then the window is dropped there.
    let rebuilt = h
        .step(Input::Displays {
            monitors: displays(1),
            bar: Default::default(),
        })
        .to_vec();
    h.window(2).monitor = 0;
    let dropped = h.event(WindowEvent::moved(wid(2))).to_vec();

    // Assert -- the rebuild lays out the first monitor around it, and the
    // drop places it there.
    assert_eq!(h.active(0), vec![1, 2]);
    assert!(places(&rebuilt, 1) && !places(&rebuilt, 2), "{rebuilt:?}");
    assert!(places(&dropped, 2), "{dropped:?}");
}

#[test]
fn a_workspace_switch_while_a_window_is_dragged_hides_it_and_ends_the_drag() {
    // Arrange -- window 1 dragged over the second monitor.
    let mut h = Harness::new(2, &[(1, 0), (2, 0), (3, 1)]);
    grab(&mut h, 1);
    h.window(1).rect.x += MON_W;
    h.window(1).monitor = 1;

    // Act -- the first monitor switches workspace, then the drag ends.
    let effects = h.action(Action::GoToWorkspace(2)).to_vec();
    let dropped = h.event(WindowEvent::moved(wid(1))).to_vec();

    // Assert -- it is hidden with the others, and stays on its workspace.
    let hides = |e: &Effect| matches!(*e, Effect::Hide { window, .. } if window == wid(1));
    assert!(effects.iter().any(hides), "{effects:?}");
    assert!(h.tiler.mouse.drag.is_none());
    assert_eq!(h.workspace(0, 0).windows(), [wid(1), wid(2)]);
    assert_eq!(h.active(1), vec![3]);
    assert!(!places(&dropped, 1), "{dropped:?}");
}

#[test]
fn a_display_change_that_parks_a_dragged_window_on_a_monocle_workspace_leaves_it_waiting() {
    // Arrange -- window 1 fills the first monitor in monocle, and the user
    // drags window 2 on the second.
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.action(Action::ToggleMonocle);
    grab(&mut h, 2);

    // Act -- the second monitor is disconnected, which brings the window
    // to the first, where the user drops it.
    let rebuilt = h.step(Input::Displays {
        monitors: displays(1),
        bar: Default::default(),
    });
    let rebuilt = rebuilt.to_vec();
    let waits = h.workspace(0, 0).waits_behind(wid(2));
    h.window(2).monitor = 0;
    let dropped = h.event(WindowEvent::moved(wid(2))).to_vec();

    // Assert -- nothing puts it behind or places it, nor gives its focus
    // away, until the drop puts it behind the monocle window.
    let behind = |e: &Effect| matches!(*e, Effect::PutBehind { window, .. } if window == wid(2));
    let focus_1 = |e: &Effect| matches!(*e, Effect::Focus { window, .. } if window == wid(1));
    assert!(
        !rebuilt.iter().any(behind) && !places(&rebuilt, 2),
        "{rebuilt:?}"
    );
    assert!(!rebuilt.iter().any(focus_1), "{rebuilt:?}");
    assert!(waits);
    assert!(
        dropped.iter().any(behind) && places(&dropped, 2),
        "{dropped:?}"
    );
}

#[test]
fn a_dragged_window_that_is_minimized_leaves_the_drag() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    grab(&mut h, 1);

    // Act -- minimized, then restored.
    h.window(1).minimized = true;
    h.event(WindowEvent::Minimized { window: wid(1) });
    h.window(1).minimized = false;
    let restored = h.event(WindowEvent::Restored { window: wid(1) }).to_vec();

    // Assert -- it is laid out again as it comes back.
    assert!(h.tiler.mouse.drag.is_none());
    assert!(places(&restored, 1), "{restored:?}");
}

#[test]
fn a_drag_that_ends_puts_the_window_back_in_its_slot() {
    // Arrange -- window 2 has its slot on the right.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let slot = h.rect_of(2);
    grab(&mut h, 2);

    // Act
    h.event(WindowEvent::moved(wid(2)));

    // Assert
    assert!(h.tiler.mouse.drag.is_none());
    assert_eq!(h.rect_of(2), slot);
}

//! What a monitor that comes back does with windows that changed while it
//! was away: one that closed or went elsewhere leaves the others their own
//! splits, each where it is once they are back, and a window displaced
//! twice keeps the split of its first home.

use crate::id::wid;
use crate::tiler::Display;
use crate::tiler::tests::{Harness, MON_H, MON_W, key};
use crate::{Action, MonitorId, Rect, WindowEvent};

use super::split_tests::{chain, connect, move_splits, moved_on_second, show};

/// Window `id` of `h` closes.
fn close(h: &mut Harness, id: u64) {
    h.desktop.windows.remove(&wid(id));
    h.event(WindowEvent::Destroyed { window: wid(id) });
}

/// Window `id` of `h` takes focus and is sent to workspace 2.
fn send_away(h: &mut Harness, id: u64) {
    h.event(WindowEvent::Focused { window: wid(id) });
    h.action(Action::SendToWorkspace(2));
}

/// The second of two monitors, alone, and so the primary one.
fn second_alone() -> Vec<Display> {
    vec![Display {
        id: MonitorId::from_raw(1),
        key: key("mon1"),
        work_area: Rect::new(MON_W, 0, MON_W, MON_H),
        primary: true,
    }]
}

#[test]
fn a_window_that_leaves_while_its_monitor_is_away_leaves_the_others_theirs() {
    // Arrange -- window 3 of four, each split moved, closed while the
    // second monitor is there, for what the others are to come back to.
    let (ids, ratios) = ([2, 3, 4, 5], [0.75, 0.25, 0.6]);
    let mut connected = moved_on_second(&ids, &ratios);
    close(&mut connected, 3);
    let ways: [fn(&mut Harness, u64); 2] = [close, send_away];

    for leave in ways {
        let mut h = moved_on_second(&ids, &ratios);

        // Act -- it closes, or goes to another workspace, while the second
        // monitor is away.
        connect(&mut h, 1);
        leave(&mut h, 3);
        connect(&mut h, 2);

        // Assert -- each of the others has its own split, where it is now.
        assert_eq!(h.active(1), vec![2, 4, 5]);
        assert_eq!(chain(&h, 1), [Some(0.75), Some(0.6)]);
        let rects = |h: &Harness| [2, 4, 5].map(|id| h.rect_of(id));
        assert_eq!(rects(&h), rects(&connected));
    }
}

#[test]
fn a_window_displaced_twice_keeps_the_split_of_its_first_home() {
    // Arrange -- windows 1 and 6 at a split of 0.3 on the first monitor,
    // and windows 2 to 5 on the second, each split moved.
    let windows = [(1, 0), (6, 0), (2, 1), (3, 1), (4, 1), (5, 1)];
    let mut h = Harness::new(2, &windows);
    move_splits(&mut h, 0, &[0.3]);
    move_splits(&mut h, 1, &[0.75, 0.25, 0.6]);

    // Act -- the first monitor leaves, then comes back as primary while the
    // second leaves in the same change, which parks windows 1 and 6 on it
    // once more, and then the second comes back too.
    show(&mut h, second_alone());
    connect(&mut h, 1);
    connect(&mut h, 2);

    // Assert
    assert_eq!((h.active(0), h.active(1)), (vec![1, 6], vec![2, 3, 4, 5]));
    assert_eq!(chain(&h, 0), [Some(0.3)]);
    assert_eq!(chain(&h, 1), [Some(0.75), Some(0.25), Some(0.6)]);
}

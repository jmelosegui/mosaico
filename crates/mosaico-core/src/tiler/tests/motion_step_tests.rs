//! The steps whose motions end together: each apply and each flush starts
//! one, so a window that falls behind ends only the motions started with
//! its own.

use crate::id::wid;
use crate::tiler::Platform;
use crate::{Action, Direction, WindowEvent};

use super::{FakePlatform, Manager, animated, placements, start, step};

/// Closes window `id`, and reports it gone.
fn close(platform: &FakePlatform, manager: &mut Manager, id: u64) {
    platform.close_window(id);
    manager.handle_event(&WindowEvent::Destroyed { window: wid(id) });
}

#[test]
fn a_window_that_falls_behind_ends_neither_the_flush_nor_the_apply_next_to_its_step() {
    // Arrange -- three monitors of two windows each. A move of window 1 is
    // held; window 4 closes, and window 3 grows into its monitor; the move
    // is flushed, and windows 1 and 2 swap; window 6 closes, and window 5
    // grows. The applications of windows 3 and 5 have stopped responding.
    let windows = [(1, 0), (2, 0), (3, 1), (4, 1), (5, 2), (6, 2)];
    let platform = FakePlatform::new(3, &windows);
    let mut manager = start(&platform, None, &animated());
    for id in [3, 5] {
        platform.edit(id, |w| w.hung = true);
    }
    let began = platform.now();
    manager.handle_action(&Action::Move(Direction::Right));
    close(&platform, &mut manager, 4);
    manager.flush_pending();
    close(&platform, &mut manager, 6);
    platform.take_calls();

    // Act -- frame steps until no window moves.
    let mut placed = Vec::new();
    while step(&platform, &mut manager) {
        for (window, _) in placements(&platform.take_calls()) {
            placed.push((window, platform.now() - began));
        }
    }

    // Assert -- windows 3 and 5 end their own steps early, and the swap
    // between them goes on to its end.
    let duration = animated().animations.spec().duration;
    let at = |id| placed.iter().find(|&&(w, _)| w == id).map(|&(_, at)| at);
    for id in [3, 5] {
        assert!(at(id) < Some(duration), "{placed:?}");
    }
    for id in [1, 2] {
        assert!(at(id) >= Some(duration), "{placed:?}");
    }
}

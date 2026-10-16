//! The held move of a window that stays tiled while a window on another
//! monitor leaves the layout, with transitions off, and again with them
//! on, once the windows have settled.

use crate::config::Config;
use crate::tiler::tests::manager_support::{animated, settle, still};

use super::*;

fn a_window_that_stays_tiled_gets_its_held_move(config: Config) {
    // Arrange -- window 3, alone on the second monitor, has never had a
    // border either.
    let windows = [(1, 0), (2, 0), (3, 1)];
    let (platform, mut manager) = without_unfocused_borders_in(config, 2, &windows);
    let left = platform.window(1).rect;
    hold_a_swap(&mut manager);

    // Act -- the window minimized is the one on the other monitor.
    platform.edit(3, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(3) });
    let mut calls = flush(&platform, &mut manager);
    settle(&platform, &mut manager);
    calls.extend(platform.take_calls());

    // Assert -- window 2 lands in the left slot.
    let window_2: Vec<Call> = calls
        .into_iter()
        .filter(|c| matches!(c, Call::Place(w, _) | Call::Repaint(w) if *w == wid(2)))
        .collect();
    assert_eq!(window_2, [Call::Place(wid(2), left), Call::Repaint(wid(2))]);
    assert_eq!(platform.window(2).rect, left);
}

#[test]
fn a_window_that_stays_tiled_still_gets_its_held_move() {
    a_window_that_stays_tiled_gets_its_held_move(still());
}

#[test]
fn a_window_that_stays_tiled_still_gets_its_held_move_with_transitions_on() {
    a_window_that_stays_tiled_gets_its_held_move(animated());
}

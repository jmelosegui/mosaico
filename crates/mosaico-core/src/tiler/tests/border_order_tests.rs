//! Where in a step each window's one border is drawn (#125): right after
//! the window is placed, so the two land together.

use super::*;

/// The step's placements, early moves, windows put behind others, borders
/// and focus changes, as `P1`, `PP1`, `PB1`, `B1`, `F1`, in order.
fn order(effects: &[Effect]) -> Vec<String> {
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Place { window, .. } => Some(format!("P{}", window.raw())),
            Effect::PrePlace { window, .. } => Some(format!("PP{}", window.raw())),
            Effect::PutBehind { window, .. } => Some(format!("PB{}", window.raw())),
            Effect::Border { window, .. } => Some(format!("B{}", window.raw())),
            Effect::Focus { window, .. } => Some(format!("F{}", window.raw())),
            _ => None,
        })
        .collect()
}

#[test]
fn a_retile_on_two_monitors_draws_each_border_right_after_its_window() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 0), (3, 1), (4, 1)]);

    // Act
    let effects = h.action(Action::Retile).to_vec();

    // Assert
    assert_eq!(
        order(&effects),
        ["P1", "B1", "P2", "B2", "P3", "B3", "P4", "B4"]
    );
}

#[test]
fn opening_a_window_draws_each_border_right_after_its_window() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);

    // Act -- the new window is moved first, then the layout runs.
    let effects = h.open(99, 0).to_vec();

    // Assert
    assert_eq!(
        order(&effects),
        [
            "PP99", "P1", "B1", "P2", "B2", "P3", "B3", "P99", "B99", "F99"
        ]
    );
}

#[test]
fn a_window_put_behind_the_monocle_window_leaves_each_border_after_its_window() {
    // Arrange -- window 1 in monocle with focus on the first monitor,
    // window 2 on the second.
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.action(Action::ToggleMonocle);

    // Act -- the second monitor goes away, so window 2 waits behind
    // window 1: put behind it, then placed at the monocle rect.
    let effects = h
        .step(Input::Displays {
            monitors: displays(1),
            bar: BarReservation::default(),
        })
        .to_vec();

    // Assert -- window 1, the front, is placed first and its border
    // comes right after it. Window 2 is put behind it and placed, its
    // border follows, and it is put behind again once it is shown.
    assert_eq!(order(&effects), ["P1", "B1", "PB2", "P2", "B2", "PB2"]);
}

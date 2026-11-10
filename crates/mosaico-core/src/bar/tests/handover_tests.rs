use super::*;
use crate::StableMonitorId;

fn id(text: &str) -> StableMonitorId {
    StableMonitorId::new(text)
}

fn top(pairs: &[(&str, i32)]) -> Reach {
    let mut reach = Reach::none();
    for (name, pixels) in pairs {
        reach.set_top(id(name), *pixels);
    }
    reach
}

#[test]
fn the_edges_the_new_bar_does_not_keep_are_held() {
    // Arrange
    let old = top(&[("m-a", 84), ("m-b", 84)]);
    let new = top(&[("m-b", 32)]);
    let now = Instant::now();

    // Act
    let handover = Handover::begin(&old, &new, now + HANDOVER);

    // Assert
    let kept = handover.map(|h| h.with(&new)).unwrap_or_default();
    assert_eq!(kept.top(&id("m-a")), 84);
    assert_eq!(kept.top(&id("m-b")), 32);
}

#[test]
fn nothing_is_held_when_the_new_bar_keeps_every_edge_the_old_one_did() {
    // Arrange
    let old = top(&[("m-a", 84)]);
    let new = top(&[("m-a", 32)]);

    // Act
    let handover = Handover::begin(&old, &new, Instant::now());

    // Assert
    assert_eq!(handover, None);
}

#[test]
fn a_monitor_the_new_bar_reaches_into_is_let_go() {
    // Arrange
    let old = top(&[("m-a", 84), ("m-b", 84)]);
    let now = Instant::now();
    let mut handover = Handover::begin(&old, &Reach::none(), now + HANDOVER).unwrap();
    let new = top(&[("m-a", 40)]);

    // Act
    handover.release(&new);

    // Assert
    let kept = handover.with(&new);
    assert_eq!(kept.top(&id("m-a")), 40);
    assert_eq!(kept.top(&id("m-b")), 84);
    assert!(!handover.over(now));
}

#[test]
fn the_hold_is_over_once_its_time_is_up_or_every_monitor_is_let_go() {
    // Arrange
    let old = top(&[("m-a", 84)]);
    let now = Instant::now();
    let mut handover = Handover::begin(&old, &Reach::none(), now + HANDOVER).unwrap();

    // Act
    let at_end = handover.over(now + HANDOVER);
    handover.release(&top(&[("m-a", 40)]));

    // Assert
    assert!(at_end);
    assert!(handover.over(now));
}

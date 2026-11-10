use crate::StableMonitorId;

use super::arbitrary::Gen;
use super::{State, Update};

#[test]
fn applying_the_diff_of_two_states_gives_the_newer() {
    for seed in 0..2000 {
        // Arrange
        let mut generator = Gen::new(seed);
        let older = generator.state();
        let newer = generator.state();

        // Act
        let mut applied = older.clone();
        applied.apply(older.diff(&newer));

        // Assert
        assert_eq!(applied, newer, "seed {seed}");
    }
}

#[test]
fn a_diff_survives_the_wire() {
    for seed in 0..500 {
        // Arrange
        let mut generator = Gen::new(seed);
        let older = generator.state();
        let newer = generator.state();
        let update = older.diff(&newer);

        // Act
        let text = serde_json::to_string(&update).unwrap_or_default();
        let read: Update = serde_json::from_str(&text).unwrap_or_default();

        // Assert
        assert_eq!(read, update, "seed {seed}");
    }
}

#[test]
fn a_state_has_no_diff_with_itself() {
    // Arrange
    let state = Gen::new(7).state();

    // Act
    let update = state.diff(&state);

    // Assert
    assert!(update.is_empty());
    assert_eq!(serde_json::to_string(&update).unwrap_or_default(), "{}");
}

#[test]
fn a_diff_carries_only_the_monitors_that_changed() {
    // Arrange
    let mut generator = Gen::new(11);
    let older = loop {
        let state = generator.state();
        if state.monitors.len() >= 2 {
            break state;
        }
    };
    let mut newer = older.clone();
    newer.monitors[1].maximized = !newer.monitors[1].maximized;

    // Act
    let update = older.diff(&newer);

    // Assert
    assert_eq!(update.monitors, vec![newer.monitors[1].clone()]);
    assert!(update.daemon.is_none() && update.theme.is_none() && update.focus.is_none());
}

#[test]
fn a_disconnected_monitor_is_removed_by_id() {
    // Arrange
    let mut generator = Gen::new(3);
    let older = loop {
        let state = generator.state();
        if !state.monitors.is_empty() {
            break state;
        }
    };
    let mut newer = older.clone();
    let gone = newer.monitors.remove(0).id;

    // Act
    let update = older.diff(&newer);

    // Assert
    assert_eq!(update.removed.monitors, vec![gone]);
}

#[test]
fn applying_puts_monitors_back_in_index_order() {
    // Arrange
    let mut state = State::default();
    let mut update = Update::default();
    for (id, index) in [("m-b", 1), ("m-a", 0)] {
        update.monitors.push(super::Monitor {
            id: StableMonitorId::new(id),
            index,
            ..Default::default()
        });
    }

    // Act
    state.apply(update);

    // Assert
    let ids: Vec<&str> = state.monitors.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["m-a", "m-b"]);
}

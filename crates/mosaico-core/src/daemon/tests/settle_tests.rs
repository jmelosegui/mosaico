use super::*;

#[test]
fn nothing_is_due_without_a_change() {
    // Arrange
    let mut settle = DisplaySettle::default();

    // Act / Assert
    assert!(!settle.take_due(Instant::now()));
    assert_eq!(settle.remaining(Instant::now()), None);
}

#[test]
fn change_is_due_once_quiet_for_the_settle_time() {
    // Arrange
    let t0 = Instant::now();
    let mut settle = DisplaySettle::default();

    // Act
    settle.note(t0);

    // Assert
    assert!(!settle.is_due(t0 + SETTLE / 2));
    assert!(settle.take_due(t0 + SETTLE));
    assert!(!settle.take_due(t0 + SETTLE * 2), "applied only once");
}

#[test]
fn a_burst_is_applied_once_after_the_last_change() {
    // Arrange
    let t0 = Instant::now();
    let mut settle = DisplaySettle::default();

    // Act -- three notifications 500 ms apart.
    settle.note(t0);
    settle.note(t0 + Duration::from_millis(500));
    settle.note(t0 + Duration::from_millis(1000));

    // Assert
    assert!(!settle.is_due(t0 + SETTLE + Duration::from_millis(500)));
    assert_eq!(
        settle.remaining(t0 + Duration::from_millis(1000)),
        Some(SETTLE)
    );
    assert!(settle.take_due(t0 + Duration::from_millis(1000) + SETTLE));
}

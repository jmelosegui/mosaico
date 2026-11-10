use super::*;

fn id(text: &str) -> StableMonitorId {
    StableMonitorId::new(text)
}

#[test]
fn a_monitor_without_an_entry_keeps_nothing_clear() {
    // Arrange
    let mut reach = Reach::none();
    reach.set_top(id("m-1"), 30);

    // Act
    let other = reach.top(&id("m-2"));

    // Assert
    assert_eq!(other, 0);
}

#[test]
fn setting_a_monitor_again_replaces_what_it_kept() {
    // Arrange
    let mut reach = Reach::none();
    reach.set_top(id("m-1"), 30);

    // Act
    reach.set_top(id("m-1"), 40);

    // Assert
    assert_eq!(reach.top(&id("m-1")), 40);
}

#[test]
fn a_negative_reach_keeps_nothing_clear() {
    // Arrange
    let mut reach = Reach::none();

    // Act
    reach.set_top(id("m-1"), -5);

    // Assert
    assert_eq!(reach.top(&id("m-1")), 0);
}

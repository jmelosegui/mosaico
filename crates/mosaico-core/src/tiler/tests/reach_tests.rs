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

#[test]
fn the_furthest_of_two_reaches_keeps_every_monitor_of_either() {
    // Arrange
    let mut a = Reach::none();
    a.set(id("m-1"), Edges::new(30, 0, 0, 0));
    let mut b = Reach::none();
    b.set(id("m-1"), Edges::new(10, 48, 0, 0));
    b.set(id("m-2"), Edges::new(0, 0, 200, 0));

    // Act
    let furthest = a.furthest(&b);

    // Assert
    assert_eq!(furthest.edges(&id("m-1")), Edges::new(30, 48, 0, 0));
    assert_eq!(furthest.edges(&id("m-2")), Edges::new(0, 0, 200, 0));
}

#[test]
fn setting_the_top_keeps_the_other_edges() {
    // Arrange
    let mut reach = Reach::none();
    reach.set(id("m-1"), Edges::new(10, 48, 5, 6));

    // Act
    reach.set_top(id("m-1"), 30);

    // Assert
    assert_eq!(reach.edges(&id("m-1")), Edges::new(30, 48, 5, 6));
}

fn display(serial: &str, x: i32, scale: f64) -> Display {
    Display {
        id: crate::MonitorId::from_raw(x as u64),
        key: crate::MonitorKey {
            stable_id: Some(serial.into()),
            ..Default::default()
        },
        bounds: Rect::new(x, 0, 1920, 1080),
        work_area: Rect::new(x, 0, 1920, 1040),
        primary: x == 0,
        name: String::new(),
        device: String::new(),
        scale,
    }
}

#[test]
fn listing_displays_numbers_them_in_order_with_the_ids_a_starting_daemon_gives() {
    // Arrange
    let displays = [display("a", 0, 1.0), display("b", 1920, 1.5)];

    // Act
    let places = MonitorPlace::list(&displays);
    let again = MonitorPlace::list(&displays);

    // Assert
    assert_eq!(places.iter().map(|p| p.index).collect::<Vec<_>>(), [0, 1]);
    assert_eq!(places[1].bounds, Rect::new(1920, 0, 1920, 1080));
    assert_eq!(places[1].scale, 1.5);
    assert_ne!(places[0].id, places[1].id);
    assert_eq!(places, again);
}

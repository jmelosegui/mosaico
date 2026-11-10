use super::*;

fn screen() -> Rect {
    Rect::new(0, 0, 1920, 1080)
}

#[test]
fn what_the_system_keeps_is_read_from_each_edge() {
    // Arrange -- a taskbar of 48 at the bottom and a sidebar of 100 on the
    // right, on a monitor to the right of another.
    let bounds = Rect::new(1920, 0, 1920, 1080);
    let work_area = Rect::new(1920, 0, 1820, 1032);

    // Act
    let kept = Edges::kept_by(&bounds, &work_area);

    // Assert
    assert_eq!(kept, Edges::new(0, 48, 0, 100));
}

#[test]
fn past_a_quarter_of_the_monitor_an_edge_is_refused() {
    // Arrange -- a quarter is 270 down and 480 across.
    let edges = Edges::new(271, 270, 480, 481);

    // Act
    let refused = edges.too_large(&screen());
    let kept = edges.within(&screen());

    // Assert
    assert_eq!(refused, [Side::Top, Side::Right]);
    assert_eq!(kept, Edges::new(0, 270, 480, 0));
}

#[test]
fn values_at_100_percent_grow_with_the_monitors_scale() {
    // Arrange
    let edges = Edges::new(32, 0, 15, 1);

    // Act
    let scaled = edges.scaled(1.5);

    // Assert -- halves round away from zero.
    assert_eq!(scaled, Edges::new(48, 0, 23, 2));
}

#[test]
fn a_scale_the_system_could_not_give_counts_as_100_percent() {
    // Arrange
    let edges = Edges::new(32, 0, 0, 0);

    // Act
    let scaled = [0.0, -1.0, f64::NAN].map(|scale| edges.scaled(scale));

    // Assert
    assert_eq!(scaled, [edges; 3]);
}

#[test]
fn the_furthest_of_two_is_taken_edge_by_edge() {
    // Arrange
    let a = Edges::new(30, 0, 10, 0);
    let b = Edges::new(20, 48, 0, 0);

    // Act
    let furthest = a.furthest(b);

    // Assert
    assert_eq!(furthest, Edges::new(30, 48, 10, 0));
}

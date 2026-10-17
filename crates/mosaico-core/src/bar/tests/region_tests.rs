use super::*;

const HEIGHT: i32 = 30;

fn widget(side: BarSide, index: usize) -> BarTarget {
    BarTarget::Widget(WidgetRef { side, index })
}

fn region(target: BarTarget, x: i32, width: i32) -> ItemRegion {
    ItemRegion {
        target,
        clickable: matches!(target, BarTarget::Workspace(_)),
        rect: Rect::new(x, 0, width, HEIGHT),
    }
}

/// A workspaces container at 0..60 with labels at 2..22 and 26..46, and
/// a pill at 70..170, drawn in that order.
fn left_side() -> Vec<ItemRegion> {
    vec![
        region(widget(BarSide::Left, 0), 0, 60),
        region(BarTarget::Workspace(0), 2, 20),
        region(BarTarget::Workspace(1), 26, 20),
        region(widget(BarSide::Left, 1), 70, 100),
    ]
}

fn target_at(regions: &[ItemRegion], x: i32, y: i32) -> Option<BarTarget> {
    hit(regions, x, y).map(|r| r.target)
}

#[test]
fn a_point_on_a_label_hits_its_workspace() {
    // Act
    let target = target_at(&left_side(), 30, 5);

    // Assert
    assert_eq!(target, Some(BarTarget::Workspace(1)));
}

#[test]
fn a_point_between_two_labels_hits_their_container() {
    // Act
    let target = target_at(&left_side(), 24, 5);

    // Assert
    assert_eq!(target, Some(widget(BarSide::Left, 0)));
}

#[test]
fn a_point_on_a_pill_hits_its_widget() {
    // Act
    let target = target_at(&left_side(), 120, HEIGHT - 1);

    // Assert
    assert_eq!(target, Some(widget(BarSide::Left, 1)));
}

#[test]
fn a_point_in_the_gap_between_two_items_hits_nothing() {
    // Act
    let target = target_at(&left_side(), 65, 5);

    // Assert
    assert_eq!(target, None);
}

#[test]
fn a_point_off_every_item_hits_nothing() {
    // Arrange
    let regions = left_side();

    // Act / Assert -- past the last item, below the bar and before it.
    assert_eq!(target_at(&regions, 500, 5), None);
    assert_eq!(target_at(&regions, 10, HEIGHT), None);
    assert_eq!(target_at(&regions, -1, 5), None);
    assert_eq!(target_at(&regions, 10, -1), None);
}

#[test]
fn an_item_takes_its_left_edge_and_not_its_right_one() {
    // Arrange
    let regions = left_side();

    // Act / Assert
    assert_eq!(target_at(&regions, 70, 5), Some(widget(BarSide::Left, 1)));
    assert_eq!(target_at(&regions, 170, 5), None);
}

#[test]
fn where_the_sides_overlap_the_right_one_drawn_last_wins() {
    // Arrange -- a long left side runs under the right side's first pill.
    let mut regions = left_side();
    regions.push(region(widget(BarSide::Right, 0), 150, 100));

    // Act
    let target = target_at(&regions, 160, 5);

    // Assert
    assert_eq!(target, Some(widget(BarSide::Right, 0)));
}

#[test]
fn two_widgets_of_the_same_kind_are_told_apart() {
    // Arrange -- two clocks, the first and third entries of the right side.
    let regions = vec![
        region(widget(BarSide::Right, 0), 300, 100),
        region(widget(BarSide::Right, 2), 190, 100),
    ];

    // Act / Assert
    assert_eq!(target_at(&regions, 350, 5), Some(widget(BarSide::Right, 0)));
    assert_eq!(target_at(&regions, 200, 5), Some(widget(BarSide::Right, 2)));
}

#[test]
fn a_bar_with_nothing_drawn_hits_nothing() {
    // Act
    let target = target_at(&[], 10, 5);

    // Assert
    assert_eq!(target, None);
}

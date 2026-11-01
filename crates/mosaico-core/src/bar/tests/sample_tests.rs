use super::sample_text;
use crate::config::{BarWidget, WidgetKind};

#[test]
fn every_widget_with_text_has_some() {
    for kind in [
        WidgetKind::Layout,
        WidgetKind::Clock,
        WidgetKind::Date,
        WidgetKind::Ram,
        WidgetKind::Cpu,
        WidgetKind::Update,
        WidgetKind::Media,
        WidgetKind::Paused,
    ] {
        assert!(
            !sample_text(&BarWidget::new(kind, true)).is_empty(),
            "{kind} reads nothing"
        );
    }
}

#[test]
fn the_clock_follows_its_format() {
    // Arrange
    let mut clock = BarWidget::new(WidgetKind::Clock, true);
    clock.format = Some("%H:%M".into());

    // Act
    let text = sample_text(&clock);

    // Assert
    assert_eq!(text, "12:57");
}

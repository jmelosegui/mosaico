use mosaico_core::config::{BarWidget, Theme, WidgetKind};

use super::{Look, sample};
use crate::icons::Icon;
use crate::pages::bar::state::BarState;

fn pills(widget: &BarWidget) -> Vec<(Option<Icon>, String)> {
    let look = Look::of(&BarState::defaults(), Theme::default());
    sample(widget, &look)
        .into_iter()
        .map(|pill| (pill.icon, pill.text))
        .collect()
}

#[test]
fn a_widget_shows_the_icon_it_is_set_to() {
    // Arrange
    let mut cpu = BarWidget::new(WidgetKind::Cpu, true);
    cpu.icon = "\u{F2DB}".into();

    // Act
    let shown = pills(&cpu);

    // Assert
    assert_eq!(shown[0].0, Some(Icon::Memory));
}

#[test]
fn a_widget_without_an_icon_shows_none() {
    // Arrange
    let mut cpu = BarWidget::new(WidgetKind::Cpu, true);
    cpu.icon = String::new();

    // Act
    let shown = pills(&cpu);

    // Assert
    assert_eq!(shown[0].0, None);
}

#[test]
fn widgets_read_as_the_bar_writes_them() {
    // Arrange
    let cpu = BarWidget::new(WidgetKind::Cpu, true);
    let ram = BarWidget::new(WidgetKind::Ram, true);

    // Act and assert
    assert_eq!(pills(&cpu)[0].1, "CPU:  12%");
    assert_eq!(pills(&ram)[0].1, "RAM: 48%");
}

#[test]
fn the_widgets_shown_only_at_times_are_previewed() {
    for kind in [WidgetKind::Update, WidgetKind::Media, WidgetKind::Paused] {
        // Arrange
        let widget = BarWidget::new(kind, true);

        // Act
        let shown = pills(&widget);

        // Assert
        assert!(
            shown.iter().any(|(_, text)| !text.is_empty()),
            "{kind} is not previewed"
        );
    }
}

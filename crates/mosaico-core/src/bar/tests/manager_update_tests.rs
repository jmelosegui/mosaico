//! What the manager draws on each bar when it refreshes.

use super::tests::{bar, left, right, start};
use crate::LayoutKind;
use crate::Rect;
use crate::bar::fakes::BarEvent::{self, Drawn};
use crate::bar::fakes::{FakeRenderer, FakeSystem};
use crate::bar::{BarContent, BarItem, BarState, CpuTimes};
use crate::config::{BarConfig, CatppuccinFlavor, Theme};

/// The labels of the pills each bar drew, by area.
fn drawn_labels(events: &[BarEvent]) -> Vec<(Rect, Vec<String>)> {
    events
        .iter()
        .filter_map(|event| match event {
            Drawn { area, content, .. } => Some((*area, labels(content))),
            _ => None,
        })
        .collect()
}

fn labels(content: &BarContent) -> Vec<String> {
    content
        .left
        .iter()
        .chain(&content.right)
        .filter_map(|item| match item {
            BarItem::Pill { label, .. } => Some(label.clone()),
            _ => None,
        })
        .collect()
}

/// The background color of each bar drawn.
fn drawn_backgrounds(events: Vec<BarEvent>) -> Vec<String> {
    events
        .into_iter()
        .filter_map(|event| match event {
            Drawn { background, .. } => Some(background),
            _ => None,
        })
        .collect()
}

#[test]
fn each_bar_shows_its_own_monitors_state() {
    // Arrange
    let renderer = FakeRenderer::default();
    let config = bar("left = []\nright = [{ type = \"layout\" }]");
    let mut manager = start(
        config,
        &[left(), right()],
        &renderer,
        &FakeSystem::default(),
    );
    renderer.take();
    let stack = BarState {
        layout: LayoutKind::VerticalStack,
        ..BarState::default()
    };

    // Act
    manager.update(&[BarState::default(), stack]);

    // Assert
    assert_eq!(
        drawn_labels(&renderer.take()),
        [
            (left(), vec!["BSP".into()]),
            (right(), vec!["VStack".into()])
        ]
    );
}

#[test]
fn a_monitor_without_a_state_shows_the_default() {
    // Arrange
    let renderer = FakeRenderer::default();
    let config = bar("left = []\nright = [{ type = \"layout\" }]");
    let mut manager = start(
        config,
        &[left(), right()],
        &renderer,
        &FakeSystem::default(),
    );
    renderer.take();
    let stack = BarState {
        layout: LayoutKind::VerticalStack,
        ..BarState::default()
    };

    // Act
    manager.update(&[stack]);

    // Assert
    assert_eq!(
        drawn_labels(&renderer.take()),
        [
            (left(), vec!["VStack".into()]),
            (right(), vec!["BSP".into()])
        ]
    );
}

#[test]
fn the_cpu_is_read_once_per_update_for_every_bar() {
    // Arrange -- 70 of the next 100 units are busy.
    let renderer = FakeRenderer::default();
    let system = FakeSystem::default();
    system.set(|m| m.cpu = Some(CpuTimes::default()));
    let config = bar("left = []\nright = [{ type = \"cpu\" }]");
    let mut manager = start(config, &[left(), right()], &renderer, &system);
    renderer.take();
    let reads = system.reads().cpu;
    system.set(|m| {
        m.cpu = Some(CpuTimes {
            idle: 30,
            total: 100,
        })
    });

    // Act
    manager.update(&[]);

    // Assert
    assert_eq!(system.reads().cpu, reads + 1);
    assert_eq!(
        drawn_labels(&renderer.take()),
        [
            (left(), vec!["CPU:  70%".into()]),
            (right(), vec!["CPU:  70%".into()])
        ]
    );
}

#[test]
fn without_a_cpu_widget_the_cpu_is_never_read() {
    // Arrange
    let system = FakeSystem::default();
    let config = bar("left = []\nright = [{ type = \"clock\" }]");
    let mut manager = start(config, &[left()], &FakeRenderer::default(), &system);

    // Act
    manager.update(&[]);
    manager.update(&[]);

    // Assert
    assert_eq!(system.reads().cpu, 0);
}

#[test]
fn reloading_with_a_cpu_widget_measures_the_cpu_from_the_reload() {
    // Arrange -- 40 of the 100 units after the reload are busy.
    let renderer = FakeRenderer::default();
    let system = FakeSystem::default();
    system.set(|m| {
        m.cpu = Some(CpuTimes {
            idle: 500,
            total: 1000,
        })
    });
    let config = bar("left = []\nright = [{ type = \"clock\" }]");
    let mut manager = start(config, &[left()], &renderer, &system);
    manager.reload(bar("left = []\nright = [{ type = \"cpu\" }]"));
    renderer.take();
    system.set(|m| {
        m.cpu = Some(CpuTimes {
            idle: 560,
            total: 1100,
        })
    });

    // Act
    manager.update(&[]);

    // Assert
    assert_eq!(
        drawn_labels(&renderer.take()),
        [(left(), vec!["CPU:  40%".into()])]
    );
}

#[test]
fn switching_the_theme_resolves_the_colors_from_the_file_again() {
    // Arrange -- Mocha filled in the background the file left empty.
    let renderer = FakeRenderer::default();
    let mut manager = start(
        BarConfig::default(),
        &[left()],
        &renderer,
        &FakeSystem::default(),
    );
    renderer.take();

    // Act
    manager.resolve_colors(Theme::Catppuccin(CatppuccinFlavor::Latte));
    manager.update(&[]);

    // Assert
    assert_eq!(drawn_backgrounds(renderer.take()), ["#eff1f5"]);
}

#[test]
fn rebuilding_under_another_theme_draws_in_its_colors() {
    // Arrange -- the bars were made under Mocha.
    let renderer = FakeRenderer::default();
    let mut manager = start(
        BarConfig::default(),
        &[left()],
        &renderer,
        &FakeSystem::default(),
    );
    renderer.take();

    // Act
    manager.rebuild_for_monitors(vec![left()], Theme::Catppuccin(CatppuccinFlavor::Latte));
    manager.update(&[]);

    // Assert
    assert_eq!(drawn_backgrounds(renderer.take()), ["#eff1f5"]);
}

#[test]
fn a_bar_that_fails_to_draw_does_not_stop_the_others() {
    // Arrange
    let renderer = FakeRenderer::default();
    let config = bar("left = []\nright = [{ type = \"layout\" }]");
    let mut manager = start(
        config,
        &[left(), right()],
        &renderer,
        &FakeSystem::default(),
    );
    renderer.take();
    renderer.fail_draws_on(left());

    // Act -- the failure is only logged, and the next refresh tries again.
    manager.update(&[]);
    let while_failing = drawn_labels(&renderer.take());
    renderer.heal_draws();
    manager.update(&[]);

    // Assert
    assert_eq!(while_failing, [(right(), vec!["BSP".into()])]);
    assert_eq!(
        drawn_labels(&renderer.take()),
        [(left(), vec!["BSP".into()]), (right(), vec!["BSP".into()])]
    );
}

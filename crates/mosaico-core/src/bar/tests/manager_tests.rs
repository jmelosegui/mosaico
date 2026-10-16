//! Which bars the manager makes, replaces and hides.

use super::*;
use crate::bar::fakes::BarEvent::{Created, Dropped, Hidden, RendererDropped};
use crate::bar::fakes::{FakeRenderer, FakeSystem};
use crate::config::CatppuccinFlavor;

#[path = "height_tests.rs"]
mod height_tests;

pub(super) const MOCHA: Theme = Theme::Catppuccin(CatppuccinFlavor::Mocha);

pub(super) fn left() -> Rect {
    Rect::new(0, 0, 1920, 1040)
}

pub(super) fn right() -> Rect {
    Rect::new(1920, 0, 1920, 1040)
}

/// A bar config written the way `bar.toml` is.
pub(super) fn bar(toml: &str) -> BarConfig {
    toml::from_str(toml).expect("a valid bar.toml")
}

/// A manager for `config` on monitors with `areas`, drawing through
/// `renderer` and reading `system`, under the Mocha theme.
pub(super) fn start(
    config: BarConfig,
    areas: &[Rect],
    renderer: &FakeRenderer,
    system: &FakeSystem,
) -> BarManager {
    BarManager::new(
        renderer.boxed(),
        Box::new(system.clone()),
        config,
        areas.to_vec(),
        MOCHA,
    )
}

#[test]
fn every_monitor_gets_a_bar() {
    // Arrange
    let renderer = FakeRenderer::default();

    // Act
    let manager = start(
        BarConfig::default(),
        &[left(), right()],
        &renderer,
        &FakeSystem::default(),
    );

    // Assert
    assert_eq!(renderer.take(), [Created(left()), Created(right())]);
    assert_eq!(manager.bar_monitor_indices(), [0, 1]);
    assert_eq!(manager.bar_height(), 64);
}

#[test]
fn only_the_monitors_in_the_config_get_a_bar() {
    // Arrange
    let renderer = FakeRenderer::default();

    // Act
    let manager = start(
        bar("monitors = [1]"),
        &[left(), right()],
        &renderer,
        &FakeSystem::default(),
    );

    // Assert
    assert_eq!(renderer.take(), [Created(right())]);
    assert_eq!(manager.bar_monitor_indices(), [1]);
}

#[test]
fn a_disabled_bar_makes_no_bars_and_takes_no_space() {
    // Arrange
    let renderer = FakeRenderer::default();

    // Act
    let manager = start(
        bar("enabled = false"),
        &[left()],
        &renderer,
        &FakeSystem::default(),
    );

    // Assert
    assert!(renderer.take().is_empty());
    assert!(manager.bar_monitor_indices().is_empty());
    assert_eq!(manager.bar_height(), 0);
}

#[test]
fn a_monitor_whose_bar_cannot_be_made_goes_without_one() {
    // Arrange
    let renderer = FakeRenderer::default();
    renderer.fail_on(left());

    // Act
    let manager = start(
        BarConfig::default(),
        &[left(), right()],
        &renderer,
        &FakeSystem::default(),
    );

    // Assert
    assert_eq!(renderer.take(), [Created(right())]);
    assert_eq!(manager.bar_monitor_indices(), [1]);
}

#[test]
fn reloading_replaces_the_bars_and_returns_the_new_height() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = start(
        BarConfig::default(),
        &[left(), right()],
        &renderer,
        &FakeSystem::default(),
    );
    renderer.take();

    // Act
    let height = manager.reload(bar("height = 40"));

    // Assert
    assert_eq!(height, 40);
    assert_eq!(
        renderer.take(),
        [
            Dropped(left()),
            Dropped(right()),
            Created(left()),
            Created(right())
        ]
    );
}

#[test]
fn rebuilding_for_new_monitors_replaces_the_bars() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = start(
        BarConfig::default(),
        &[left(), right()],
        &renderer,
        &FakeSystem::default(),
    );
    renderer.take();

    // Act
    manager.rebuild_for_monitors(vec![right()], MOCHA);

    // Assert
    assert_eq!(
        renderer.take(),
        [Dropped(left()), Dropped(right()), Created(right())]
    );
    assert_eq!(manager.bar_monitor_indices(), [0]);
}

#[test]
fn hiding_hides_every_bar() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = start(
        BarConfig::default(),
        &[left(), right()],
        &renderer,
        &FakeSystem::default(),
    );
    renderer.take();

    // Act
    manager.hide_all();

    // Assert
    assert_eq!(renderer.take(), [Hidden(left()), Hidden(right())]);
}

#[test]
fn the_bars_are_dropped_before_their_renderer() {
    // Arrange
    let renderer = FakeRenderer::default();
    let manager = start(
        BarConfig::default(),
        &[left(), right()],
        &renderer,
        &FakeSystem::default(),
    );
    renderer.take();

    // Act
    drop(manager);

    // Assert
    assert_eq!(
        renderer.take(),
        [Dropped(left()), Dropped(right()), RendererDropped]
    );
}

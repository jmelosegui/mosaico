//! When the bars let the pointer through to the windows under them.

use crate::bar::fakes::{FakeRenderer, FakeSystem};
use crate::bar::{BarManager, BarState};
use crate::config::BarConfig;

use super::super::tests::{bar, left, right, start};

fn manager(config: BarConfig, renderer: &FakeRenderer) -> BarManager {
    start(config, &[left(), right()], renderer, &FakeSystem::default())
}

/// Two monitors, the second with a maximized window when `maximized`.
fn states(maximized: bool) -> [BarState; 2] {
    let full = BarState {
        maximized,
        ..BarState::default()
    };
    [BarState::default(), full]
}

#[test]
fn every_bar_takes_the_pointer_from_its_first_refresh() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = manager(BarConfig::default(), &renderer);

    // Act
    manager.update(&states(false));

    // Assert
    assert_eq!(
        renderer.take_switches(),
        [(left(), false), (right(), false)]
    );
}

#[test]
fn a_bar_over_a_maximized_window_lets_the_pointer_through() {
    // Arrange -- bars are made letting it through, so the second stays.
    let renderer = FakeRenderer::default();
    let mut manager = manager(BarConfig::default(), &renderer);

    // Act
    manager.update(&states(true));

    // Assert
    assert_eq!(renderer.take_switches(), [(left(), false)]);
}

#[test]
fn a_bar_takes_the_pointer_again_once_the_window_is_gone() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = manager(BarConfig::default(), &renderer);
    manager.update(&states(false));
    manager.update(&states(true));
    assert_eq!(renderer.take_switches().last(), Some(&(right(), true)));

    // Act
    manager.update(&states(false));

    // Assert
    assert_eq!(renderer.take_switches(), [(right(), false)]);
}

#[test]
fn a_bar_is_switched_only_when_it_changes() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = manager(BarConfig::default(), &renderer);
    manager.update(&states(false));
    renderer.take_switches();

    // Act
    manager.update(&states(false));

    // Assert
    assert!(renderer.take_switches().is_empty());
}

#[test]
fn with_the_mouse_off_every_bar_lets_the_pointer_through() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = manager(bar("[mouse]\nenabled = false"), &renderer);

    // Act
    manager.update(&states(false));

    // Assert -- as made, so nothing is switched.
    assert!(renderer.take_switches().is_empty());
}

#[test]
fn a_bar_that_could_not_be_switched_is_asked_again_on_the_next_refresh() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = manager(BarConfig::default(), &renderer);
    renderer.fail_switches_on(left());
    manager.update(&states(false));
    renderer.take_switches();
    renderer.heal_switches();

    // Act
    manager.update(&states(false));

    // Assert
    assert_eq!(renderer.take_switches(), [(left(), false)]);
}

#[test]
fn bars_made_again_take_the_pointer_from_their_first_refresh() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = manager(BarConfig::default(), &renderer);
    manager.update(&states(false));
    renderer.take_switches();

    // Act
    manager.reload(BarConfig::default());
    manager.update(&states(false));

    // Assert
    assert_eq!(
        renderer.take_switches(),
        [(left(), false), (right(), false)]
    );
}

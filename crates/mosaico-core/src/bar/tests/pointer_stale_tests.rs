//! What the pointer did on bars that are gone, or have nothing drawn on
//! them to click.

use std::time::Instant;

use crate::Action;
use crate::action::Origin;
use crate::bar::fakes::FakeRenderer;
use crate::bar::{BarId, BarState, Button};
use crate::config::BarConfig;

use super::super::tests::{MOCHA, bar, left, right};
use super::support::{bar_on, click, drawn, label, press, release};

#[test]
fn a_release_on_a_bar_made_again_since_the_press_is_dropped() {
    // Arrange -- the bars are made again between the press and release.
    let renderer = FakeRenderer::default();
    let mut manager = drawn(BarConfig::default(), &renderer);
    let old = bar_on(&renderer, 0);
    let now = Instant::now();
    manager.pointer(old, press(Button::Left, label(1)), now);
    manager.reload(BarConfig::default());
    manager.resolve_colors(MOCHA);
    manager.update(&[BarState::default(), BarState::default()]);
    let new = bar_on(&renderer, 0);

    // Act
    let stale = manager.pointer(old, release(Button::Left, label(1)), now);
    let fresh = manager.pointer(new, release(Button::Left, label(1)), now);

    // Assert -- the new bar never saw the press.
    assert_ne!(old, new);
    assert_eq!((stale, fresh), (None, None));
}

#[test]
fn a_click_on_a_bar_made_for_new_monitors_names_the_new_bar() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(BarConfig::default(), &renderer);
    let old = bar_on(&renderer, 0);
    manager.rebuild_for_monitors(vec![right()], MOCHA);
    manager.update(&[BarState::default()]);
    let new = bar_on(&renderer, 0);

    // Act
    let stale = click(&mut manager, old, label(1), label(1));
    let fresh = click(&mut manager, new, label(1), label(1));

    // Assert
    assert_eq!(stale, None);
    assert_eq!(fresh, Some((Action::GoToWorkspace(2), Origin::Bar(0))));
}

#[test]
fn a_bar_whose_draw_failed_has_nothing_to_click() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(BarConfig::default(), &renderer);
    let first = bar_on(&renderer, 0);
    renderer.fail_draws_on(left());
    manager.update(&[BarState::default(), BarState::default()]);

    // Act
    let asked = click(&mut manager, first, label(1), label(1));

    // Assert
    assert_eq!(asked, None);
}

#[test]
fn a_hidden_bar_has_nothing_to_click() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(BarConfig::default(), &renderer);
    let first = bar_on(&renderer, 0);
    manager.hide_all();

    // Act
    let asked = click(&mut manager, first, label(1), label(1));

    // Assert
    assert_eq!(asked, None);
}

#[test]
fn with_the_mouse_off_a_click_asks_for_nothing() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(bar("[mouse]\nenabled = false"), &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let asked = click(&mut manager, first, label(1), label(1));

    // Assert
    assert_eq!(asked, None);
}

#[test]
fn a_bar_no_monitor_has_is_dropped() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(BarConfig::default(), &renderer);
    let first = bar_on(&renderer, 0);
    let elsewhere = BarId {
        monitor: 5,
        ..first
    };

    // Act
    let asked = click(&mut manager, elsewhere, label(1), label(1));

    // Assert
    assert_eq!(asked, None);
}

#[test]
fn each_bar_is_made_with_its_monitor_and_every_set_with_a_new_generation() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(BarConfig::default(), &renderer);

    // Act
    manager.reload(BarConfig::default());

    // Assert
    let ids = renderer.ids();
    let made: Vec<(usize, u64)> = ids.iter().map(|id| (id.monitor, id.generation)).collect();
    assert_eq!(made, [(0, 1), (1, 1), (0, 2), (1, 2)]);
}

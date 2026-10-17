//! When the turns waiting on a bar are dropped, and what is kept of them
//! from one burst to the next.

use std::time::Instant;

use crate::Action;
use crate::action::Origin;
use crate::bar::fakes::FakeRenderer;
use crate::bar::{Button, WHEEL_QUIET};
use crate::config::BarConfig;

use super::super::tests::{MOCHA, left, right};
use super::support::{bar_on, click, drawn_with, label, press, showing, turn};

#[test]
fn a_click_on_a_label_drops_the_turns_waiting_on_its_bar() {
    // Arrange -- a notch down over monitor 0's bar.
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let first = bar_on(&renderer, 0);
    let last = turn(&mut manager, first, &[1.0], Instant::now());

    // Act -- a click on 5 before the wheel goes quiet, which the daemon
    // carries out.
    let asked = click(&mut manager, first, label(4), label(4));
    manager.update(&showing(&[4, 0]));
    let switches = manager.take_wheel(last + WHEEL_QUIET);

    // Assert -- monitor 0 stays on 5.
    assert_eq!(asked, Some((Action::GoToWorkspace(5), Origin::Bar(0))));
    assert_eq!(manager.wheel_due(), None);
    assert!(switches.is_empty());
}

#[test]
fn any_press_on_a_bar_drops_the_turns_waiting_there() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let first = bar_on(&renderer, 0);
    let last = turn(&mut manager, first, &[1.0], Instant::now());

    // Act -- a press of the right button, which asks for nothing.
    manager.pointer(first, press(Button::Right, label(2)), last);

    // Assert
    assert_eq!(manager.wheel_due(), None);
    assert!(manager.take_wheel(last + WHEEL_QUIET).is_empty());
}

#[test]
fn a_switch_made_some_other_way_drops_the_turns_waiting() {
    // Arrange -- a notch down over monitor 0's bar.
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let first = bar_on(&renderer, 0);
    let last = turn(&mut manager, first, &[1.0], Instant::now());

    // Act -- a key takes monitor 0 to 5 before the wheel goes quiet.
    manager.update(&showing(&[4, 0]));
    let switches = manager.take_wheel(last + WHEEL_QUIET);

    // Assert
    assert!(switches.is_empty());
}

#[test]
fn turns_after_a_switch_made_some_other_way_step_from_where_it_went() {
    // Arrange -- a key took monitor 0 to 5 while a notch waited.
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let first = bar_on(&renderer, 0);
    let last = turn(&mut manager, first, &[1.0], Instant::now());
    manager.update(&showing(&[4, 0]));
    manager.take_wheel(last + WHEEL_QUIET);

    // Act
    let later = turn(&mut manager, first, &[1.0], last + WHEEL_QUIET * 2);
    let switches = manager.take_wheel(later + WHEEL_QUIET);

    // Assert
    assert_eq!(switches, [(Action::GoToWorkspace(6), Origin::Bar(0))]);
}

#[test]
fn fractions_turned_slowly_switch_once_they_make_a_notch() {
    // Arrange -- half a notch, applied once the wheel went quiet.
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let first = bar_on(&renderer, 0);
    let last = turn(&mut manager, first, &[0.5], Instant::now());
    let none = manager.take_wheel(last + WHEEL_QUIET);

    // Act -- the other half, later.
    let later = turn(&mut manager, first, &[0.5], last + WHEEL_QUIET * 2);
    let switches = manager.take_wheel(later + WHEEL_QUIET);

    // Assert
    assert!(none.is_empty());
    assert_eq!(switches, [(Action::GoToWorkspace(2), Origin::Bar(0))]);
}

#[test]
fn a_fraction_left_over_does_not_outlive_the_bars() {
    // Arrange -- half a notch over monitor 0's bar, applied.
    let renderer = FakeRenderer::default();
    let mut manager = drawn_with(BarConfig::default(), &showing(&[0, 0]), &renderer);
    let last = turn(&mut manager, bar_on(&renderer, 0), &[0.5], Instant::now());
    manager.take_wheel(last + WHEEL_QUIET);

    // Act -- the bars are made again, and the other half turns over the
    // new one.
    manager.rebuild_for_monitors(vec![left(), right()], MOCHA);
    manager.update(&showing(&[0, 0]));
    let later = turn(
        &mut manager,
        bar_on(&renderer, 0),
        &[0.5],
        last + WHEEL_QUIET * 2,
    );

    // Assert
    assert!(manager.take_wheel(later + WHEEL_QUIET).is_empty());
}

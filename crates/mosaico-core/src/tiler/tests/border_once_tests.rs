//! A step draws each window's border at most once (#125).

use std::collections::HashMap;

use crate::Direction;
use crate::config::Color;

use super::*;

const FOCUSED: Color = Color {
    r: 0x11,
    g: 0x11,
    b: 0x11,
};
const UNFOCUSED: Color = Color {
    r: 0x22,
    g: 0x22,
    b: 0x22,
};

/// A harness on one monitor whose focused and unfocused borders differ.
fn colored(windows: &[(u64, usize)]) -> Harness {
    let mut settings = Settings::default();
    settings.borders.colors.focused = "#111111".into();
    settings.borders.colors.unfocused = "#222222".into();
    Harness::with_settings(settings, 1, windows)
}

/// How many times the step decides each window's border.
fn borders_per_window(effects: &[Effect]) -> HashMap<u64, usize> {
    let mut counts = HashMap::new();
    for effect in effects {
        if let Effect::Border { window, .. } = effect {
            *counts.entry(window.raw()).or_insert(0) += 1;
        }
    }
    counts
}

/// The color the step draws `window`'s border in.
fn color_of(effects: &[Effect], window: u64) -> Option<Color> {
    effects.iter().find_map(|e| match e {
        Effect::Border {
            window: w,
            style: Some(style),
        } if w.raw() == window => Some(style.color),
        _ => None,
    })
}

/// Asserts that no window's border is decided twice, and that each one
/// comes after its window's last placement, around the slot it lands in.
fn assert_each_border_once_after_its_window(effects: &[Effect]) {
    let twice: Vec<u64> = borders_per_window(effects)
        .into_iter()
        .filter(|&(_, n)| n > 1)
        .map(|(w, _)| w)
        .collect();
    assert!(twice.is_empty(), "{twice:?} bordered twice: {effects:?}");
    for (i, effect) in effects.iter().enumerate() {
        let Effect::Border {
            window,
            style: Some(style),
        } = effect
        else {
            continue;
        };
        let placed = effects.iter().enumerate().rev().find_map(|(j, e)| match e {
            Effect::Place {
                window: w, rect, ..
            } if w == window => Some((j, *rect)),
            _ => None,
        });
        if let Some((j, rect)) = placed {
            assert!(j < i, "{window} bordered before it is placed: {effects:?}");
            assert_eq!(rect, style.rect, "{window} bordered at another slot");
        }
    }
}

#[test]
fn opening_a_window_next_to_another_draws_each_border_once() {
    // Arrange
    let mut h = colored(&[(1, 0)]);

    // Act
    let effects = h.open(99, 0).to_vec();

    // Assert -- in the colors the step ends with.
    assert_eq!(
        borders_per_window(&effects),
        HashMap::from([(1, 1), (99, 1)])
    );
    assert_each_border_once_after_its_window(&effects);
    assert_eq!(color_of(&effects, 99), Some(FOCUSED));
    assert_eq!(color_of(&effects, 1), Some(UNFOCUSED));
}

#[test]
fn opening_a_window_next_to_three_draws_each_border_once() {
    // Arrange
    let mut h = colored(&[(1, 0), (2, 0), (3, 0)]);

    // Act
    let effects = h.open(99, 0).to_vec();

    // Assert
    assert_eq!(
        borders_per_window(&effects),
        HashMap::from([(1, 1), (2, 1), (3, 1), (99, 1)])
    );
    assert_each_border_once_after_its_window(&effects);
}

#[test]
fn a_retile_draws_each_border_once() {
    // Arrange
    let mut h = colored(&[(1, 0), (2, 0), (3, 0)]);

    // Act
    let effects = h.action(Action::Retile).to_vec();

    // Assert
    assert_eq!(
        borders_per_window(&effects),
        HashMap::from([(1, 1), (2, 1), (3, 1)])
    );
    assert_each_border_once_after_its_window(&effects);
}

#[test]
fn a_focus_change_draws_each_border_once() {
    // Arrange
    let mut h = colored(&[(1, 0), (2, 0), (3, 0)]);

    // Act
    let effects = h.action(Action::Focus(Direction::Right)).to_vec();

    // Assert
    assert_each_border_once_after_its_window(&effects);
    assert_eq!(color_of(&effects, 2), Some(FOCUSED));
    assert_eq!(color_of(&effects, 1), Some(UNFOCUSED));
}

#[test]
fn a_workspace_switch_draws_each_border_once() {
    // Arrange
    let mut h = colored(&[(1, 0), (2, 0), (3, 0)]);

    // Act -- send a window away, which follows it there, come back, and
    // go there again.
    let sent = h.action(Action::SendToWorkspace(2)).to_vec();
    let back = h.action(Action::GoToWorkspace(1)).to_vec();
    let again = h.action(Action::GoToWorkspace(2)).to_vec();

    // Assert
    for effects in [sent, back, again] {
        assert!(
            effects.iter().any(|e| matches!(e, Effect::Border { .. })),
            "a switch that draws no border: {effects:?}"
        );
        assert_each_border_once_after_its_window(&effects);
    }
}

//! A window that starts floating comes in front of everything else on its
//! workspace, whether it had focus or not.

use crate::config::{RuleAction, WindowRule};
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{Effect, Input};
use crate::{Action, WindowEvent, WindowId};

/// Which window the last border drawn around `window` goes behind, when
/// not its own.
fn border_behind(h: &Harness, window: u64) -> Option<WindowId> {
    h.effects
        .iter()
        .rev()
        .find_map(|e| match e {
            Effect::Border { window: w, style } if *w == wid(window) => {
                Some(style.as_ref().and_then(|s| s.behind))
            }
            _ => None,
        })
        .flatten()
}

#[test]
fn the_border_of_a_focused_tile_goes_behind_the_floating_windows() {
    // Arrange -- windows 2 and 3 float, 3 in front.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    for id in [2, 3] {
        h.event(WindowEvent::Focused { window: wid(id) });
        h.action(Action::ToggleFloat);
    }

    // Act
    h.event(WindowEvent::Focused { window: wid(1) });

    // Assert
    assert_eq!(border_behind(&h, 1), Some(wid(2)));
}

#[test]
fn a_tile_floated_by_the_user_comes_in_front_of_the_floating_windows() {
    // Arrange -- window 2 floats, and tile 3 has focus behind it.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);
    h.event(WindowEvent::Focused { window: wid(3) });

    // Act
    h.action(Action::ToggleFloat);

    // Assert
    assert_eq!(h.workspace(0, 0).floating(), &[wid(2), wid(3)]);
    assert!(h.desktop.in_front(wid(3), wid(2)));
    assert!(h.desktop.in_front(wid(2), wid(1)));
}

#[test]
fn a_tile_a_rules_reload_floats_comes_in_front_of_the_tiles_and_floating_windows() {
    // Arrange -- window 3 floats, and tile 1 has focus behind it.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.event(WindowEvent::Focused { window: wid(3) });
    h.action(Action::ToggleFloat);
    h.event(WindowEvent::Focused { window: wid(1) });
    if let Some(facts) = h.window(2).facts.as_mut() {
        facts.class = "Tool".into();
    }

    // Act
    h.step(Input::Rules(vec![WindowRule {
        match_class: Some("Tool".into()),
        match_title: None,
        match_path: None,
        action: RuleAction::Float,
    }]));

    // Assert
    assert_eq!(h.workspace(0, 0).floating(), &[wid(3), wid(2)]);
    assert!(h.desktop.in_front(wid(2), wid(3)));
    assert!(h.desktop.in_front(wid(3), wid(1)));
}

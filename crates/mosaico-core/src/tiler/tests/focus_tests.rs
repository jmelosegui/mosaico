use crate::Direction;
use crate::config::Color;

use super::*;

fn following_cursor() -> Settings {
    Settings {
        mouse_follows_focus: true,
        ..Settings::default()
    }
}

#[test]
fn focus_right_goes_to_the_neighbor() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);

    // Act
    h.action(Action::Focus(Direction::Right));

    // Assert
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
    assert_eq!(h.focus_request(), Some(2));
}

#[test]
fn focus_right_at_the_edge_overflows_to_the_next_monitor() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);

    // Act
    h.action(Action::Focus(Direction::Right));

    // Assert
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
}

#[test]
fn focus_left_at_the_leftmost_monitor_stays_put() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);

    // Act
    let effects = h.action(Action::Focus(Direction::Left)).to_vec();

    // Assert
    assert!(effects.is_empty());
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn keyboard_focus_moves_the_cursor_when_it_follows_focus() {
    // Arrange
    let mut h = Harness::with_settings(following_cursor(), 1, &[(1, 0), (2, 0)]);

    // Act
    let effects = h.action(Action::Focus(Direction::Right)).to_vec();

    // Assert
    assert!(effects.contains(&Effect::Focus {
        window: wid(2),
        move_cursor: true
    }));
}

#[test]
fn the_focus_border_follows_focus_immediately() {
    // Arrange
    let mut settings = Settings::default();
    settings.borders.colors.focused = "#111111".into();
    settings.borders.colors.unfocused = "#222222".into();
    let mut h = Harness::with_settings(settings, 1, &[(1, 0), (2, 0)]);

    // Act
    let effects = h.action(Action::Focus(Direction::Right)).to_vec();

    // Assert -- 2 gets the focused color, 1 the unfocused one.
    let color_of = |w: u64| {
        effects.iter().rev().find_map(|e| match e {
            Effect::Border { window, style } if *window == wid(w) => {
                style.as_ref().map(|s| s.color)
            }
            _ => None,
        })
    };
    assert_eq!(color_of(2), Color::from_hex("#111111"));
    assert_eq!(color_of(1), Color::from_hex("#222222"));
}

#[test]
fn entering_an_empty_monitor_clears_focus_and_parks_it() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0)]);

    // Act
    h.action(Action::Focus(Direction::Right));

    // Assert
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert_eq!(h.tiler.focused_window(), None);
}

#[test]
fn a_focus_restore_elsewhere_does_not_move_a_parked_monitor() {
    // Arrange -- the user went to the empty second monitor, pointer
    // and all, and a launcher took and released focus.
    let mut h = Harness::new(2, &[(1, 0)]);
    h.action(Action::Focus(Direction::Right));
    h.desktop.cursor = Some((MON_W + 500, 500));

    // Act -- the OS hands focus back to the window on monitor 0.
    h.event(WindowEvent::Focused { window: wid(1) });
    h.open(2, 0);

    // Assert -- the new window opens where the user is.
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert_eq!(h.active(1), vec![2]);
}

#[test]
fn clicking_the_other_monitor_ends_the_parking() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0)]);
    h.action(Action::Focus(Direction::Right));

    // Act -- the pointer is on monitor 0, where focus went.
    h.desktop.cursor = Some((500, 500));
    h.event(WindowEvent::Focused { window: wid(1) });

    // Assert
    assert_eq!(h.tiler.focused_monitor(), 0);
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn parking_expires_as_a_backstop() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0)]);
    h.action(Action::Focus(Direction::Right));
    h.desktop.cursor = Some((MON_W + 500, 500));

    // Act
    h.advance(std::time::Duration::from_secs(31));
    h.event(WindowEvent::Focused { window: wid(1) });

    // Assert
    assert_eq!(h.tiler.focused_monitor(), 0);
}

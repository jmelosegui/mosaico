//! Window effects carried out against a platform that records what it
//! is asked to do.

use crate::config::CornerStyle;

use super::window_recorder::{Call, Recorder};
use super::*;

/// Where window 1 is on the desktop. Its center is (420, 340).
const AT: Rect = Rect {
    x: 100,
    y: 100,
    width: 640,
    height: 480,
};

/// A slot window 1 is not in yet.
const SLOT: Rect = Rect {
    x: 0,
    y: 0,
    width: 960,
    height: 1080,
};

/// A desktop with window 1 open at [`AT`].
fn desktop() -> FakeDesktop {
    let mut window = FakeWindow::app(0);
    window.rect = AT;
    let mut desktop = FakeDesktop::default();
    desktop.windows.insert(wid(1), window);
    desktop
}

fn place(rect: Rect) -> Effect {
    Effect::Place {
        window: wid(1),
        rect,
        timing: Timing::Now,
    }
}

fn focus(move_cursor: bool) -> Effect {
    Effect::Focus {
        window: wid(1),
        move_cursor,
    }
}

/// Carries out `effects` in order and returns what the platform was
/// asked to do.
fn apply(effects: &[Effect], desktop: &FakeDesktop, windows: &Recorder) -> Vec<Call> {
    for effect in effects {
        apply_window_effect(effect, desktop, windows);
    }
    windows.calls.take()
}

#[test]
fn a_placement_moves_the_window_then_repaints_it() {
    // Arrange
    let windows = Recorder::default();

    // Act
    let calls = apply(&[place(SLOT)], &desktop(), &windows);

    // Assert
    assert_eq!(calls, [Call::Place(wid(1), SLOT), Call::Repaint(wid(1))]);
}

#[test]
fn a_window_already_in_place_is_left_alone() {
    // Arrange
    let windows = Recorder::default();

    // Act
    let calls = apply(&[place(AT)], &desktop(), &windows);

    // Assert
    assert!(calls.is_empty(), "{calls:?}");
}

#[test]
fn a_maximized_window_is_placed_even_when_already_in_place() {
    // Arrange -- placing it is what takes it out of maximized state.
    let mut desktop = desktop();
    desktop.windows.get_mut(&wid(1)).unwrap().maximized = true;
    let windows = Recorder::default();

    // Act
    let calls = apply(&[place(AT)], &desktop, &windows);

    // Assert
    assert_eq!(calls, [Call::Place(wid(1), AT), Call::Repaint(wid(1))]);
}

#[test]
fn a_window_is_repainted_even_when_placing_it_fails() {
    // Arrange
    let windows = Recorder::default();
    windows.failing.set(true);

    // Act
    let calls = apply(&[place(SLOT)], &desktop(), &windows);

    // Assert
    assert_eq!(calls, [Call::Place(wid(1), SLOT), Call::Repaint(wid(1))]);
}

#[test]
fn a_window_not_shown_yet_is_pre_placed_without_a_repaint() {
    // Arrange
    let windows = Recorder::default();
    let effect = Effect::PrePlace {
        window: wid(1),
        rect: SLOT,
    };

    // Act
    let calls = apply(&[effect], &desktop(), &windows);

    // Assert
    assert_eq!(calls, [Call::PrePlace(wid(1), SLOT)]);
}

#[test]
fn focus_puts_the_pointer_at_the_window_center_only_when_asked() {
    // Arrange
    let windows = Recorder::default();

    // Act
    let calls = apply(&[focus(true), focus(false)], &desktop(), &windows);

    // Assert
    assert_eq!(
        calls,
        [
            Call::Focus(wid(1)),
            Call::MoveCursor(420, 340),
            Call::Focus(wid(1)),
        ]
    );
}

#[test]
fn the_pointer_follows_focus_even_when_the_focus_change_is_refused() {
    // Arrange
    let windows = Recorder::default();
    windows.failing.set(true);

    // Act
    let calls = apply(&[focus(true)], &desktop(), &windows);

    // Assert
    assert_eq!(calls, [Call::Focus(wid(1)), Call::MoveCursor(420, 340)]);
}

#[test]
fn the_pointer_stays_put_for_a_window_that_is_gone() {
    // Arrange
    let windows = Recorder::default();

    // Act
    let calls = apply(
        &[Effect::CenterCursor { window: wid(9) }],
        &desktop(),
        &windows,
    );

    // Assert
    assert!(calls.is_empty(), "{calls:?}");
}

#[test]
fn hiding_and_showing_tell_the_platform_how() {
    // Arrange
    let windows = Recorder::default();
    let window = wid(1);
    let (cloak, minimize) = (HidingBehaviour::Cloak, HidingBehaviour::Minimize);

    // Act
    let calls = apply(
        &[
            Effect::Hide { window, how: cloak },
            Effect::Show { window, how: cloak },
            Effect::Hide {
                window,
                how: minimize,
            },
            Effect::Show {
                window,
                how: minimize,
            },
            Effect::Reveal { window },
        ],
        &desktop(),
        &windows,
    );

    // Assert
    assert_eq!(
        calls,
        [
            Call::Hide(window, cloak),
            Call::Show(window, cloak),
            Call::Hide(window, minimize),
            Call::Show(window, minimize),
            Call::Reveal(window),
        ]
    );
}

#[test]
fn closing_minimizing_and_repainting_reach_the_platform() {
    // Arrange
    let windows = Recorder::default();
    let window = wid(1);

    // Act
    let calls = apply(
        &[
            Effect::Close { window },
            Effect::Minimize { window },
            Effect::Repaint { window },
        ],
        &desktop(),
        &windows,
    );

    // Assert
    assert_eq!(
        calls,
        [
            Call::Close(window),
            Call::Minimize(window),
            Call::Repaint(window),
        ]
    );
}

#[test]
fn borders_and_decorations_are_left_to_the_caller() {
    // Arrange
    let windows = Recorder::default();
    let window = wid(1);

    // Act
    let calls = apply(
        &[
            Effect::Decorate {
                window,
                corners: CornerStyle::Round,
            },
            Effect::Border {
                window,
                style: None,
            },
            Effect::DropBorder { window },
            Effect::Undecorate { window },
        ],
        &desktop(),
        &windows,
    );

    // Assert
    assert!(calls.is_empty(), "{calls:?}");
}

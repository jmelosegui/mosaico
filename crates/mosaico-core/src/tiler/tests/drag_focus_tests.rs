//! Focus as the user starts dragging a window, as the owner chose while
//! testing: the window grabbed is the focused one from the start, on its
//! monitor, its border in the focused or the monocle style and the one it
//! left unfocused, without the platform asked to focus it or the pointer
//! moved.

use crate::config::Color;
use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{Effect, Settings};
use crate::{Action, WindowEvent};

/// Settings with the focused, unfocused and monocle borders in colors of
/// their own.
fn colored() -> Settings {
    let mut settings = Settings::with_gap(8);
    let colors = &mut settings.borders.colors;
    colors.focused = "#ff0000".into();
    colors.unfocused = "#00ff00".into();
    colors.monocle = "#0000ff".into();
    settings
}

const FOCUSED: Color = Color { r: 255, g: 0, b: 0 };
const UNFOCUSED: Color = Color { r: 0, g: 255, b: 0 };
const MONOCLE: Color = Color { r: 0, g: 0, b: 255 };

/// The color `effects` draw the border of window `id` in last, if they
/// draw it.
fn color_of(effects: &[Effect], id: u64) -> Option<Color> {
    effects.iter().rev().find_map(|e| match e {
        Effect::Border { window, style } if *window == wid(id) => style.as_ref().map(|s| s.color),
        _ => None,
    })
}

/// Whether `effects` ask the platform to move focus or the pointer.
fn focus_or_pointer(effects: &[Effect]) -> bool {
    let moves = |e: &Effect| matches!(e, Effect::Focus { .. } | Effect::CenterCursor { .. });
    effects.iter().any(moves)
}

/// The user grabs window `id` by its title bar, where it is, with no
/// report that it took focus.
pub(super) fn grab_unreported(h: &mut Harness, id: u64) -> Vec<Effect> {
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(h.rect_of(id)),
        ..DragStart::default()
    };
    let window = wid(id);
    h.event(WindowEvent::MoveStarted { window, start }).to_vec()
}

#[test]
fn a_window_grabbed_without_focus_takes_it_in_the_tiler() {
    // Arrange -- window 1 has focus.
    let mut h = Harness::with_settings(colored(), 1, &[(1, 0), (2, 0)]);

    // Act
    let effects = grab_unreported(&mut h, 2);

    // Assert -- window 2 is focused, with the focused border, and window 1
    // has the unfocused one, without focus or the pointer moved.
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
    assert_eq!(color_of(&effects, 2), Some(FOCUSED));
    assert_eq!(color_of(&effects, 1), Some(UNFOCUSED));
    assert!(!focus_or_pointer(&effects), "{effects:?}");
}

#[test]
fn a_window_grabbed_on_another_monitor_takes_that_monitor_with_focus() {
    // Arrange -- window 1, on the first monitor, has focus.
    let mut h = Harness::with_settings(colored(), 2, &[(1, 0), (2, 1)]);

    // Act
    grab_unreported(&mut h, 2);

    // Assert
    assert_eq!(h.tiler.focused_monitor(), 1);
}

#[test]
fn the_monocle_window_grabbed_takes_the_monocle_border() {
    // Arrange -- window 1 fills the first monitor in monocle, and window
    // 3, on the second, has focus.
    let windows = [(1, 0), (2, 0), (3, 1)];
    let mut h = Harness::with_settings(colored(), 2, &windows);
    h.action(Action::ToggleMonocle);
    h.event(WindowEvent::Focused { window: wid(3) });

    // Act
    let effects = grab_unreported(&mut h, 1);

    // Assert
    assert_eq!(color_of(&effects, 1), Some(MONOCLE));
    assert_eq!(color_of(&effects, 3), Some(UNFOCUSED));
}

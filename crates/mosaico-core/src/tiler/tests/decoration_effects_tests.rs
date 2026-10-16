//! Border and corner effects carried out against a platform that
//! records what it is asked to draw.

use crate::config::{Color, CornerStyle};
use crate::log::Level;
use crate::tiler::decoration_effects::border_failure_level;
use crate::{Decorations, Error};

use super::*;

/// A change the platform was asked to make.
#[derive(Debug, Clone, PartialEq)]
enum Call {
    ShowBorder(WindowId, BorderStyle),
    HideBorder(WindowId),
    DropBorder(WindowId),
    SetCorners(WindowId, CornerStyle),
    ResetCorners(WindowId),
}

/// A platform that records every change it is asked to make.
#[derive(Default)]
struct Painter {
    calls: Vec<Call>,
}

impl Painter {
    fn record(&mut self, call: Call) -> crate::Result<()> {
        self.calls.push(call);
        Ok(())
    }
}

impl Decorations for Painter {
    fn show_border(&mut self, window: WindowId, style: &BorderStyle) -> crate::Result<()> {
        self.record(Call::ShowBorder(window, style.clone()))
    }

    fn hide_border(&mut self, window: WindowId) -> crate::Result<()> {
        self.record(Call::HideBorder(window))
    }

    fn drop_border(&mut self, window: WindowId) -> crate::Result<()> {
        self.record(Call::DropBorder(window))
    }

    fn set_corners(&mut self, window: WindowId, corners: CornerStyle) -> crate::Result<()> {
        self.record(Call::SetCorners(window, corners))
    }

    fn reset_corners(&mut self, window: WindowId) -> crate::Result<()> {
        self.record(Call::ResetCorners(window))
    }
}

/// Where window 1 is.
const AT: Rect = Rect {
    x: 100,
    y: 100,
    width: 640,
    height: 480,
};

fn style() -> BorderStyle {
    BorderStyle {
        rect: AT,
        color: Color {
            r: 0x11,
            g: 0x22,
            b: 0x33,
        },
        width: 4,
        radius: 8,
    }
}

/// Carries out `effects` in order and returns what the platform was
/// asked to do.
fn apply(effects: &[Effect], decorations: &mut Painter) -> Vec<Call> {
    for effect in effects {
        apply_decoration_effect(effect, decorations);
    }
    std::mem::take(&mut decorations.calls)
}

#[test]
fn a_border_is_drawn_hidden_and_dropped_through_the_platform() {
    // Arrange
    let mut decorations = Painter::default();
    let window = wid(1);

    // Act
    let calls = apply(
        &[
            Effect::Border {
                window,
                style: Some(style()),
            },
            Effect::Border {
                window,
                style: None,
            },
            Effect::DropBorder { window },
        ],
        &mut decorations,
    );

    // Assert
    assert_eq!(
        calls,
        [
            Call::ShowBorder(window, style()),
            Call::HideBorder(window),
            Call::DropBorder(window),
        ]
    );
}

#[test]
fn corners_are_set_and_given_back_through_the_platform() {
    // Arrange
    let mut decorations = Painter::default();
    let window = wid(1);

    // Act
    let calls = apply(
        &[
            Effect::Decorate {
                window,
                corners: CornerStyle::Round,
            },
            Effect::Undecorate { window },
        ],
        &mut decorations,
    );

    // Assert
    assert_eq!(
        calls,
        [
            Call::SetCorners(window, CornerStyle::Round),
            Call::ResetCorners(window),
        ]
    );
}

#[test]
fn window_changes_are_left_to_apply_window_effect() {
    // Arrange
    let mut decorations = Painter::default();
    let window = wid(1);
    let how = HidingBehaviour::Cloak;

    // Act
    let calls = apply(
        &[
            Effect::Place {
                window,
                rect: AT,
                timing: Timing::Now,
            },
            Effect::PrePlace { window, rect: AT },
            Effect::Hide { window, how },
            Effect::Show { window, how },
            Effect::Reveal { window },
            Effect::Focus {
                window,
                move_cursor: true,
            },
            Effect::CenterCursor { window },
            Effect::Close { window },
            Effect::Minimize { window },
            Effect::Repaint { window },
        ],
        &mut decorations,
    );

    // Assert
    assert!(calls.is_empty(), "{calls:?}");
}

#[test]
fn a_border_that_cannot_be_created_is_a_warning() {
    // Arrange
    let error = Error::platform("create a border window", "out of memory");

    // Act
    let level = border_failure_level(&error);

    // Assert
    assert_eq!(level, Level::Warn);
}

#[test]
fn a_platform_without_borders_is_only_logged_for_debugging() {
    // Arrange -- it says so on every redraw.
    let error = Error::Unsupported("drawing focus borders");

    // Act
    let level = border_failure_level(&error);

    // Assert
    assert_eq!(level, Level::Debug);
}

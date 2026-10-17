//! What the pointer does on a bar, as a platform reports it, and where it
//! reports it.

use std::str::FromStr;

/// Identifies a bar while it exists.
///
/// [`BarManager`](super::BarManager) gives each bar its id as it makes
/// it: the monitor the bar is on, and the generation of the bars it was
/// made with. The bars are made again after every reload of `bar.toml`
/// and every display change, each time with a new generation, so what
/// the pointer did on a bar that is gone is never taken for something it
/// did on the bar now on that monitor. Only the manager makes ids, and
/// tests standing in for it; a platform hands them back as it was given
/// them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BarId {
    /// The index of the bar's monitor, in the order the monitors were
    /// given to the manager, which is the tiler's.
    pub monitor: usize,
    /// The generation of the bars the bar was made with.
    pub generation: u64,
}

/// A mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    /// The primary button, usually the left one.
    Left,
    /// The secondary button, usually the right one.
    Right,
    /// The middle button, or a press of the wheel.
    Middle,
}

impl FromStr for Button {
    type Err = String;

    /// The button named `s`, as `[[mouse.click]]` names them: `"left"`,
    /// `"right"` or `"middle"`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "middle" => Ok(Self::Middle),
            _ => Err(format!("unknown button: {s}")),
        }
    }
}

/// Something the pointer did on a bar. Positions are in bar coordinates:
/// pixels from the bar's top left corner, and past its edges while it
/// holds the pointer for a press.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerEvent {
    /// A button went down over the bar.
    Press {
        /// The button.
        button: Button,
        /// Pixels from the bar's left edge.
        x: i32,
        /// Pixels from the bar's top edge.
        y: i32,
    },
    /// A button went up after going down over the bar, wherever the
    /// pointer is then.
    Release {
        /// The button.
        button: Button,
        /// Pixels from the bar's left edge.
        x: i32,
        /// Pixels from the bar's top edge.
        y: i32,
    },
    /// The wheel turned over the bar.
    Wheel {
        /// How far, in notches: positive down or to the right, negative up
        /// or to the left. A touchpad reports fractions of a notch.
        notches: f32,
        /// Pixels from the bar's left edge.
        x: i32,
        /// Pixels from the bar's top edge.
        y: i32,
    },
    /// The pointer left the bar, or the bar lost hold of it during a
    /// press, so no release will follow.
    Leave,
}

/// Where a bar reports what the pointer does on it.
///
/// The daemon passes one to every bar it makes, through
/// [`BarRenderer::create`](super::BarRenderer::create).
///
/// # Contract for platform implementations
///
/// - Platforms call [`send`](Self::send) on the thread that made the
///   bar, while they handle the pointer's input, under the bar's own id.
/// - It returns at once. It never draws, hides or drops a bar, so a
///   platform may call it from inside its handling of the bar's input.
pub trait PointerSink {
    /// Reports that the pointer did `event` on the bar `bar`.
    fn send(&self, bar: BarId, event: PointerEvent);
}

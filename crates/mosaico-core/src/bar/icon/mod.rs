//! The line drawings a widget can show as its icon.
//!
//! A widget's `icon` setting holds a Nerd Font character. When it is one
//! of the characters below, every bar and the settings window draw the
//! same lines for it with [`mask`], instead of relying on a font that
//! may not be installed. Any other character is shown as text.

mod drawings;
mod mask;

pub use mask::mask;

/// One stroke of a line drawing, on a grid 24 units wide and tall.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stroke {
    /// A line through the points.
    Open(&'static [(f32, f32)]),
    /// A line through the points and back to the first.
    Closed(&'static [(f32, f32)]),
    /// A circle around a center, with a radius.
    Circle((f32, f32), f32),
    /// An arc around a center, with a radius, from one angle to another,
    /// in radians.
    Arc((f32, f32), f32, f32, f32),
}

/// An icon a widget can show, drawn in lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BarIcon {
    /// Four squares.
    Grid,
    /// A window with a title bar.
    Window,
    /// A clock face.
    Clock,
    /// A calendar page.
    Calendar,
    /// A memory stick.
    Memory,
    /// A processor chip.
    Processor,
    /// An arrow down onto a line.
    Download,
    /// Two bars.
    Pause,
    /// A pair of notes.
    Music,
    /// A frame split into a column and two rows.
    Columns,
}

impl BarIcon {
    /// Every icon, in the order the settings window offers them.
    pub const ALL: [Self; 10] = [
        Self::Grid,
        Self::Window,
        Self::Clock,
        Self::Calendar,
        Self::Memory,
        Self::Processor,
        Self::Download,
        Self::Pause,
        Self::Music,
        Self::Columns,
    ];

    /// The character a widget's `icon` setting holds for this icon.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Grid => "\u{F009}",
            Self::Window => "\u{F2D0}",
            Self::Clock => "\u{F017}",
            Self::Calendar => "\u{F073}",
            Self::Memory => "\u{F2DB}",
            Self::Processor => "\u{F085}",
            Self::Download => "\u{F019}",
            Self::Pause => "\u{F04C}",
            Self::Music => "\u{F001}",
            Self::Columns => "\u{F0DB}",
        }
    }

    /// The name the settings window shows for this icon.
    pub fn name(self) -> &'static str {
        match self {
            Self::Grid => "Grid",
            Self::Window => "Window",
            Self::Clock => "Clock",
            Self::Calendar => "Calendar",
            Self::Memory => "Memory",
            Self::Processor => "Processor",
            Self::Download => "Download",
            Self::Pause => "Pause",
            Self::Music => "Music",
            Self::Columns => "Columns",
        }
    }

    /// The icon drawn for `glyph`, when it is one of [`BarIcon::ALL`].
    pub fn from_glyph(glyph: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|icon| icon.glyph() == glyph)
    }
}

#[cfg(test)]
#[path = "../tests/icon_tests.rs"]
mod tests;

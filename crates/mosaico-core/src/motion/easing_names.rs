//! The names easings are written with in `config.toml`.

use std::fmt;
use std::str::FromStr;

use crate::{Error, Result};

use super::{CubicBezier, Easing};

impl Easing {
    /// The presets, in the order a list of easings shows them.
    pub const PRESETS: [Easing; 5] = [
        Self::EaseOutCubic,
        Self::EaseOutQuad,
        Self::EaseOutExpo,
        Self::EaseInOutCubic,
        Self::Linear,
    ];

    /// The easing's name for people, such as `Ease out (cubic)`. Every
    /// cubic Bezier is a `Custom curve`.
    pub fn label(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::EaseOutQuad => "Ease out (quad)",
            Self::EaseOutCubic => "Ease out (cubic)",
            Self::EaseInOutCubic => "Ease in and out (cubic)",
            Self::EaseOutExpo => "Ease out (expo)",
            Self::CubicBezier(_) => "Custom curve",
        }
    }
}

impl FromStr for Easing {
    type Err = Error;

    /// Parses the name of a preset, such as `ease-out-cubic`, or a curve
    /// written as in CSS, `cubic-bezier(x1, y1, x2, y2)`, with all four
    /// numbers within 0 to 1 and spaces allowed inside the parentheses.
    /// Names are lowercase.
    fn from_str(name: &str) -> Result<Self> {
        let easing = match name {
            "linear" => Self::Linear,
            "ease-out-quad" => Self::EaseOutQuad,
            "ease-out-cubic" => Self::EaseOutCubic,
            "ease-in-out-cubic" => Self::EaseInOutCubic,
            "ease-out-expo" => Self::EaseOutExpo,
            _ => return parse_bezier(name).map(Self::CubicBezier),
        };
        Ok(easing)
    }
}

/// Presets by their names, and a cubic Bezier with each number written
/// the shortest way that reads back as the same number, such as
/// `cubic-bezier(0.2, 0, 0, 1)`.
impl fmt::Display for Easing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Linear => "linear",
            Self::EaseOutQuad => "ease-out-quad",
            Self::EaseOutCubic => "ease-out-cubic",
            Self::EaseInOutCubic => "ease-in-out-cubic",
            Self::EaseOutExpo => "ease-out-expo",
            Self::CubicBezier(curve) => {
                let [x1, y1, x2, y2] = curve.points();
                return write!(f, "cubic-bezier({x1}, {y1}, {x2}, {y2})");
            }
        };
        f.write_str(name)
    }
}

impl TryFrom<String> for Easing {
    type Error = Error;

    fn try_from(name: String) -> Result<Self> {
        name.parse()
    }
}

impl From<Easing> for String {
    fn from(easing: Easing) -> Self {
        easing.to_string()
    }
}

/// Reads `cubic-bezier(x1, y1, x2, y2)`.
fn parse_bezier(name: &str) -> Result<CubicBezier> {
    let Some(numbers) = name
        .strip_prefix("cubic-bezier(")
        .and_then(|rest| rest.strip_suffix(')'))
    else {
        return Err(unknown(name));
    };
    let numbers: Option<Vec<f64>> = numbers.split(',').map(number).collect();
    let Some([x1, y1, x2, y2]) = numbers.and_then(|n| <[f64; 4]>::try_from(n).ok()) else {
        return Err(invalid(name, "expected four numbers"));
    };
    // Outside 0 to 1, time would go back along the curve or the window
    // would overshoot.
    CubicBezier::new(x1, y1, x2, y2)
        .ok_or_else(|| invalid(name, "all four numbers must be within 0 to 1"))
}

/// One number of a cubic Bezier, if it is a finite one.
fn number(text: &str) -> Option<f64> {
    text.trim().parse().ok().filter(|n: &f64| n.is_finite())
}

/// The error for a name that is not an easing.
fn unknown(name: &str) -> Error {
    let message = format!(
        "unknown easing {name:?} (expected linear, ease-out-quad, ease-out-cubic, \
         ease-in-out-cubic, ease-out-expo or cubic-bezier(x1, y1, x2, y2))"
    );
    Error::config(None, message)
}

/// The error for a cubic Bezier written wrong.
fn invalid(name: &str, problem: &str) -> Error {
    Error::config(None, format!("invalid easing {name:?}: {problem}"))
}

#[cfg(test)]
#[path = "tests/easing_names_tests.rs"]
mod tests;

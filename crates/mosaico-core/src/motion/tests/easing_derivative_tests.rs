use std::f64::consts::LN_2;

use super::super::{EXPO_END, Easing};
use super::{PRESETS, every_easing, times};

#[test]
fn the_derivative_is_the_slope_of_the_progress() {
    for easing in every_easing() {
        // Arrange -- a cubic Bezier is solved to within 1e-7 of the time,
        // which a longer step keeps out of the difference.
        let step = if PRESETS.contains(&easing) {
            1e-6
        } else {
            1e-3
        };
        let tolerance = if PRESETS.contains(&easing) {
            1e-5
        } else {
            2e-3
        };

        for t in times(20).filter(|t| (0.05..=0.95).contains(t)) {
            // Act
            let slope = easing.derivative(t);
            let rise = easing.progress(t + step) - easing.progress(t - step);

            // Assert
            let numeric = rise / (2.0 * step);
            assert!(
                (slope - numeric).abs() <= tolerance * (1.0 + numeric.abs()),
                "{easing} at {t}: {slope}, numerically {numeric}"
            );
        }
    }
}

#[test]
fn the_derivative_at_the_ends_is_the_slope_of_the_curve_there() {
    // Arrange -- the curves that ease out arrive at rest, and their start
    // is as steep as their formulas say.
    let cases = [
        (Easing::Linear, 1.0, 1.0),
        (Easing::EaseOutQuad, 2.0, 0.0),
        (Easing::EaseOutCubic, 3.0, 0.0),
        (Easing::EaseInOutCubic, 0.0, 0.0),
        (
            Easing::EaseOutExpo,
            10.0 * LN_2 / EXPO_END,
            10.0 * LN_2 / 1024.0 / EXPO_END,
        ),
    ];

    for (easing, start, end) in cases {
        // Act
        let (at_start, at_end) = (easing.derivative(0.0), easing.derivative(1.0));

        // Assert
        assert!(
            (at_start - start).abs() < 1e-12,
            "{easing} starts at {at_start}"
        );
        assert!((at_end - end).abs() < 1e-12, "{easing} ends at {at_end}");
    }
}

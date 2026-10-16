use super::*;

#[path = "easing_derivative_tests.rs"]
mod easing_derivative_tests;

/// Every preset, and curves of the user's own: CSS ease, a steep ease in
/// and out, one that stalls halfway and one that starts vertical.
fn every_easing() -> Vec<Easing> {
    let mut easings = PRESETS.to_vec();
    for [x1, y1, x2, y2] in [
        [0.25, 0.1, 0.25, 1.0],
        [0.7, 0.0, 0.3, 1.0],
        [0.5, 1.0, 0.5, 0.0],
        [0.0, 0.55, 0.45, 1.0],
    ] {
        easings.push(Easing::CubicBezier(
            CubicBezier::new(x1, y1, x2, y2).unwrap(),
        ));
    }
    easings
}

const PRESETS: [Easing; 5] = [
    Easing::Linear,
    Easing::EaseOutQuad,
    Easing::EaseOutCubic,
    Easing::EaseInOutCubic,
    Easing::EaseOutExpo,
];

/// `count + 1` times evenly spread from 0 to 1.
fn times(count: u32) -> impl Iterator<Item = f64> {
    (0..=count).map(move |i| f64::from(i) / f64::from(count))
}

#[test]
fn every_easing_starts_at_exactly_zero_and_ends_at_exactly_one() {
    for easing in every_easing() {
        // Act
        let (start, end) = (easing.progress(0.0), easing.progress(1.0));

        // Assert
        assert_eq!((start, end), (0.0, 1.0), "{easing}");
    }
}

#[test]
fn time_outside_the_motion_is_clamped_to_its_ends() {
    for easing in every_easing() {
        // Act
        let before = easing.progress(-0.5);
        let after = easing.progress(1.5);
        let unknown = easing.progress(f64::NAN);

        // Assert
        assert_eq!((before, after, unknown), (0.0, 1.0, 0.0), "{easing}");
        assert_eq!(easing.derivative(-0.5), easing.derivative(0.0), "{easing}");
        assert_eq!(easing.derivative(1.5), easing.derivative(1.0), "{easing}");
    }
}

#[test]
fn no_easing_goes_back_or_past_its_target() {
    for easing in every_easing() {
        // Act
        let progress: Vec<f64> = times(1000).map(|t| easing.progress(t)).collect();

        // Assert
        let back = progress.windows(2).position(|pair| pair[1] < pair[0]);
        assert_eq!(back, None, "{easing} goes back after step {back:?}");
        let past = progress.iter().find(|p| !(0.0..=1.0).contains(*p));
        assert_eq!(past, None, "{easing} leaves 0 to 1");
    }
}

#[test]
fn the_presets_follow_their_formulas() {
    // Arrange
    let cases = [
        (Easing::Linear, 0.3, 0.3),
        (Easing::EaseOutQuad, 0.5, 0.75),
        (Easing::EaseOutCubic, 0.5, 0.875),
        (Easing::EaseInOutCubic, 0.25, 0.0625),
        (Easing::EaseInOutCubic, 0.5, 0.5),
        (Easing::EaseInOutCubic, 0.75, 0.9375),
        // (1 - 2^-5) / (1 - 2^-10) = 992 / 1023.
        (Easing::EaseOutExpo, 0.5, 992.0 / 1023.0),
    ];

    for (easing, t, expected) in cases {
        // Act
        let progress = easing.progress(t);

        // Assert
        assert!(
            (progress - expected).abs() < 1e-12,
            "{easing} at {t}: {progress}"
        );
    }
}

use super::*;

fn curve(x1: f64, y1: f64, x2: f64, y2: f64) -> CubicBezier {
    CubicBezier::new(x1, y1, x2, y2).unwrap()
}

#[test]
fn a_curve_along_the_diagonal_is_linear() {
    // Arrange -- control points a third and two thirds along the diagonal
    // make time and progress the same polynomial of the parameter.
    let curves = [
        curve(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0),
        curve(0.0, 0.0, 1.0, 1.0),
    ];

    for line in curves {
        for t in [0.01, 0.2, 0.5, 0.77, 0.99] {
            // Act
            let progress = line.progress(t);

            // Assert
            assert!((progress - t).abs() < 1e-6, "{line:?} at {t}: {progress}");
        }
    }
}

#[test]
fn the_curve_passes_through_its_own_points() {
    // Arrange -- CSS ease, ease-in and ease-out, a curve that stalls
    // halfway and one that starts vertical.
    let curves = [
        curve(0.25, 0.1, 0.25, 1.0),
        curve(0.42, 0.0, 1.0, 1.0),
        curve(0.0, 0.0, 0.58, 1.0),
        curve(0.5, 1.0, 0.5, 0.0),
        curve(0.0, 0.55, 0.45, 1.0),
    ];

    for bezier in curves {
        let [x1, y1, x2, y2] = bezier.points();
        for u in [0.05, 0.2, 0.35, 0.5, 0.65, 0.8, 0.95] {
            // Act -- the point the curve's own parameter gives.
            let (x, y) = (coordinate(x1, x2, u), coordinate(y1, y2, u));
            let progress = bezier.progress(x);

            // Assert
            assert!(
                (progress - y).abs() < 1e-6,
                "{bezier:?} at {x}: {progress}, expected {y}"
            );
        }
    }
}

#[test]
fn css_ease_matches_values_worked_out_exactly() {
    // Arrange -- solved in rational numbers by bisection to 2^-120.
    let ease = curve(0.25, 0.1, 0.25, 1.0);
    let standard = curve(0.2, 0.0, 0.0, 1.0);
    let cases = [
        (ease, 0.1, 0.094_796_305_716_043_25),
        (ease, 0.25, 0.408_510_591_355_395_86),
        (ease, 0.5, 0.802_403_387_584_857),
        (ease, 0.75, 0.960_458_978_348_974_1),
        (standard, 0.1, 0.156_25),
        (standard, 0.5, 0.877_833_604_809_760_9),
        (standard, 0.9, 0.996_459_040_511_671_3),
    ];

    for (bezier, t, expected) in cases {
        // Act
        let progress = bezier.progress(t);

        // Assert
        assert!(
            (progress - expected).abs() < 1e-6,
            "{bezier:?} at {t}: {progress}"
        );
    }
}

#[test]
fn a_curve_too_flat_for_newton_is_solved_by_bisection() {
    // Arrange -- time stands still at the middle of this curve, where
    // Newton's method divides by nothing: it is vertical there.
    let bezier = curve(1.0, 0.0, 0.0, 1.0);

    for t in [0.3, 0.49, 0.5, 0.51, 0.7] {
        // Act
        let u = bezier.solve(t);

        // Assert -- the parameter found reaches the time asked for.
        let x = coordinate(1.0, 0.0, u);
        assert!((x - t).abs() < EPSILON, "at {t}: reached {x}");
        assert!((0.0..=1.0).contains(&u), "at {t}: {u}");
    }
}

#[test]
fn a_vertical_curve_has_a_steep_but_finite_slope() {
    // Arrange -- the first starts vertical, the second is vertical in the
    // middle, and the third has both derivatives vanish at the start,
    // where its slope is y2 / x2.
    let starts_vertical = curve(0.0, 0.55, 0.45, 1.0);
    let middle_vertical = curve(1.0, 0.0, 0.0, 1.0);
    let flat_start = curve(0.0, 0.0, 0.58, 1.0);

    // Act
    let slopes = [
        starts_vertical.slope(0.0),
        middle_vertical.slope(0.5),
        flat_start.slope(0.0),
    ];

    // Assert
    assert_eq!(slopes[0], MAX_SLOPE);
    assert_eq!(slopes[1], MAX_SLOPE);
    assert!((slopes[2] - 1.0 / 0.58).abs() < 1e-3, "{}", slopes[2]);
}

#[test]
fn control_points_outside_zero_to_one_are_refused() {
    // Arrange -- time going back, overshooting either end, and numbers
    // that are not numbers.
    let refused = [
        [-0.1, 0.0, 0.5, 1.0],
        [0.5, 0.0, 1.1, 1.0],
        [0.68, -0.6, 0.32, 1.0],
        [0.68, 0.0, 0.32, 1.6],
        [f64::NAN, 0.0, 0.5, 1.0],
        [0.5, f64::INFINITY, 0.5, 1.0],
    ];

    for [x1, y1, x2, y2] in refused {
        // Act / Assert
        assert_eq!(
            CubicBezier::new(x1, y1, x2, y2),
            None,
            "{:?}",
            [x1, y1, x2, y2]
        );
    }
    let corners = CubicBezier::new(0.0, 1.0, 1.0, 0.0).map(CubicBezier::points);
    assert_eq!(corners, Some([0.0, 1.0, 1.0, 0.0]));
}

use super::*;

fn bezier(x1: f64, y1: f64, x2: f64, y2: f64) -> Easing {
    Easing::CubicBezier(CubicBezier::new(x1, y1, x2, y2).unwrap())
}

#[test]
fn the_presets_are_written_with_their_config_names() {
    // Arrange
    let names = [
        (Easing::Linear, "linear"),
        (Easing::EaseOutQuad, "ease-out-quad"),
        (Easing::EaseOutCubic, "ease-out-cubic"),
        (Easing::EaseInOutCubic, "ease-in-out-cubic"),
        (Easing::EaseOutExpo, "ease-out-expo"),
        (bezier(0.2, 0.0, 0.0, 1.0), "cubic-bezier(0.2, 0, 0, 1)"),
    ];

    for (easing, name) in names {
        // Act / Assert
        assert_eq!(easing.to_string(), name);
        assert_eq!(name.parse::<Easing>().unwrap(), easing, "{name}");
    }
}

#[test]
fn every_easing_reads_back_as_what_was_written() {
    // Arrange -- including numbers with no short decimal form.
    let easings = [
        Easing::default(),
        bezier(0.1 + 0.2, 0.6, 1.0 / 3.0, 0.1 + 0.7),
        bezier(0.0, 1e-9, 1.0, 0.123_456_789),
    ];

    for easing in easings {
        // Act
        let back: Easing = easing.to_string().parse().unwrap();

        // Assert
        assert_eq!(back, easing, "{easing}");
    }
}

#[test]
fn a_cubic_bezier_may_have_spaces_inside_its_parentheses() {
    // Act
    let easing: Easing = "cubic-bezier( 0.2 ,0.0,  0.0 , 1.0 )".parse().unwrap();

    // Assert
    assert_eq!(easing, bezier(0.2, 0.0, 0.0, 1.0));
}

#[test]
fn an_unknown_easing_is_refused_with_the_names_that_work() {
    for name in [
        "bouncy",
        "Linear",
        "ease-out",
        " linear",
        "cubic-bezier (0, 0, 1, 1)",
    ] {
        // Act
        let error = name.parse::<Easing>().unwrap_err();

        // Assert
        let message = error.to_string();
        assert!(
            matches!(error, Error::Config { path: None, .. }),
            "{error:?}"
        );
        assert!(
            message.contains(&format!("unknown easing {name:?}")),
            "{message}"
        );
        assert!(message.contains("ease-in-out-cubic"), "{message}");
    }
}

#[test]
fn a_cubic_bezier_written_wrong_says_what_is_wrong() {
    // Arrange -- among them curves that CSS takes but that overshoot.
    const OUTSIDE: &str = "all four numbers must be within 0 to 1";
    let cases = [
        ("cubic-bezier(0.2, 0, 0)", "expected four numbers"),
        ("cubic-bezier(0.2, 0, 0, 1, 1)", "expected four numbers"),
        ("cubic-bezier(a, 0, 0, 1)", "expected four numbers"),
        ("cubic-bezier(0.2, inf, 0, 1)", "expected four numbers"),
        ("cubic-bezier()", "expected four numbers"),
        ("cubic-bezier(1.5, 0, 0, 1)", OUTSIDE),
        ("cubic-bezier(0, 0, -0.5, 1)", OUTSIDE),
        ("cubic-bezier(0.68, -0.6, 0.32, 1.6)", OUTSIDE),
        ("cubic-bezier(0.2, 0, 0, 1.01)", OUTSIDE),
    ];

    for (name, problem) in cases {
        // Act
        let error = name.parse::<Easing>().unwrap_err();

        // Assert
        assert_eq!(
            error.to_string(),
            format!("invalid easing {name:?}: {problem}")
        );
    }
}

#[test]
fn an_easing_is_stored_as_its_name() {
    // Arrange
    let easing = bezier(0.2, 0.0, 0.0, 1.0);

    // Act
    let json = serde_json::to_string(&easing).unwrap();
    let back: Easing = serde_json::from_str(&json).unwrap();
    let refused = serde_json::from_str::<Easing>("\"bouncy\"").unwrap_err();

    // Assert
    assert_eq!(json, "\"cubic-bezier(0.2, 0, 0, 1)\"");
    assert_eq!(back, easing);
    assert!(refused.to_string().contains("unknown easing"), "{refused}");
}

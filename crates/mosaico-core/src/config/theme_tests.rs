use super::*;

#[test]
fn default_theme_config_is_catppuccin_mocha() {
    let tc = ThemeConfig::default();
    assert_eq!(tc.name, "catppuccin");
    assert_eq!(tc.flavor, "mocha");
    assert_eq!(tc.resolve(), Theme::Catppuccin(CatppuccinFlavor::Mocha));
}

#[test]
fn resolve_all_catppuccin_flavors() {
    let cases = [
        ("mocha", CatppuccinFlavor::Mocha),
        ("macchiato", CatppuccinFlavor::Macchiato),
        ("frappe", CatppuccinFlavor::Frappe),
        ("latte", CatppuccinFlavor::Latte),
    ];
    for (flavor, expected) in cases {
        let tc = ThemeConfig {
            name: "catppuccin".into(),
            flavor: flavor.into(),
        };
        assert_eq!(tc.resolve(), Theme::Catppuccin(expected), "flavor {flavor}");
    }
}

#[test]
fn resolve_rose_pine_flavors() {
    let cases = [
        ("main", RosePineFlavor::Main),
        ("moon", RosePineFlavor::Moon),
        ("dawn", RosePineFlavor::Dawn),
    ];
    for (flavor, expected) in cases {
        let tc = ThemeConfig {
            name: "rose-pine".into(),
            flavor: flavor.into(),
        };
        assert_eq!(tc.resolve(), Theme::RosePine(expected), "flavor {flavor}");
    }
}

#[test]
fn resolve_tokyo_night_flavors() {
    let cases = [
        ("night", TokyoNightFlavor::Night),
        ("storm", TokyoNightFlavor::Storm),
        ("day", TokyoNightFlavor::Day),
    ];
    for (flavor, expected) in cases {
        let tc = ThemeConfig {
            name: "tokyo-night".into(),
            flavor: flavor.into(),
        };
        assert_eq!(tc.resolve(), Theme::TokyoNight(expected), "flavor {flavor}");
    }
}

#[test]
fn rose_pine_name_aliases() {
    for name in ["rose-pine", "rosepine", "rose_pine", "Rose-Pine"] {
        let tc = ThemeConfig {
            name: name.into(),
            flavor: "main".into(),
        };
        assert_eq!(tc.resolve(), Theme::RosePine(RosePineFlavor::Main));
    }
}

#[test]
fn tokyo_night_name_aliases() {
    for name in ["tokyo-night", "tokyonight", "tokyo_night", "Tokyo-Night"] {
        let tc = ThemeConfig {
            name: name.into(),
            flavor: "night".into(),
        };
        assert_eq!(tc.resolve(), Theme::TokyoNight(TokyoNightFlavor::Night));
    }
}

#[test]
fn resolve_is_case_insensitive() {
    let tc = ThemeConfig {
        name: "Catppuccin".into(),
        flavor: "Latte".into(),
    };
    assert_eq!(tc.resolve(), Theme::Catppuccin(CatppuccinFlavor::Latte));
}

#[test]
fn unknown_theme_falls_back_to_default() {
    let tc = ThemeConfig {
        name: "tokyo".into(), // missing the -night
        flavor: "night".into(),
    };
    assert_eq!(tc.resolve(), Theme::default());
}

#[test]
fn unknown_flavor_falls_back_to_family_default() {
    let tc = ThemeConfig {
        name: "catppuccin".into(),
        flavor: "espresso".into(),
    };
    assert_eq!(tc.resolve(), Theme::Catppuccin(CatppuccinFlavor::Mocha));

    let tc = ThemeConfig {
        name: "rose-pine".into(),
        flavor: "midnight".into(),
    };
    assert_eq!(tc.resolve(), Theme::RosePine(RosePineFlavor::Main));
}

#[test]
fn theme_config_round_trips_through_toml() {
    let tc = ThemeConfig {
        name: "rose-pine".into(),
        flavor: "moon".into(),
    };
    let toml_str = toml::to_string(&tc).unwrap();
    let parsed: ThemeConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed, tc);
}

#[test]
fn mocha_bar_colors_match_catppuccin() {
    let c = Theme::Catppuccin(CatppuccinFlavor::Mocha).bar_colors();
    assert_eq!(c.background, "#1e1e2e");
    assert_eq!(c.foreground, "#89b4fa");
    assert_eq!(c.active_workspace, "#435375");
    assert_eq!(c.pill_border, "#89b4fa");
}

#[test]
fn latte_is_light_theme() {
    let c = Theme::Catppuccin(CatppuccinFlavor::Latte).bar_colors();
    assert_eq!(c.background, "#eff1f5");
    assert_eq!(c.foreground, "#1e66f5");
}

#[test]
fn rose_pine_main_bar_features_iris_and_rose() {
    // The bar foreground and the focused window border share
    // Iris (purple) so the focused window visually ties into the
    // bar's lead color.
    let c = Theme::RosePine(RosePineFlavor::Main).bar_colors();
    assert_eq!(c.background, "#191724"); // Base
    assert_eq!(c.foreground, "#c4a7e7"); // Iris
    assert_eq!(c.accent, "#ebbcba"); // Rose
    assert_eq!(
        Theme::RosePine(RosePineFlavor::Main).border_focused(),
        c.foreground
    );
}

#[test]
fn tokyo_night_bar_uses_blue_accent() {
    let c = Theme::TokyoNight(TokyoNightFlavor::Night).bar_colors();
    assert_eq!(c.background, "#1a1b26");
    assert_eq!(c.foreground, "#7aa2f7");
}

#[test]
fn border_focused_matches_per_flavor() {
    assert_eq!(
        Theme::Catppuccin(CatppuccinFlavor::Mocha).border_focused(),
        "#89b4fa"
    );
    // Rose Pine matches its bar foreground (Iris).
    assert_eq!(
        Theme::RosePine(RosePineFlavor::Main).border_focused(),
        "#c4a7e7"
    );
    assert_eq!(
        Theme::TokyoNight(TokyoNightFlavor::Night).border_focused(),
        "#7aa2f7"
    );
}

#[test]
fn rose_pine_focus_border_matches_bar_foreground() {
    // Every Rose Pine flavor's focused border equals its bar
    // foreground so the highlight is consistent across the UI.
    for flavor in [
        RosePineFlavor::Main,
        RosePineFlavor::Moon,
        RosePineFlavor::Dawn,
    ] {
        let theme = Theme::RosePine(flavor);
        assert_eq!(
            theme.border_focused(),
            theme.bar_colors().foreground,
            "flavor {flavor:?}"
        );
    }
}

#[test]
fn border_unfocused_distinct_from_focused_for_every_theme() {
    let themes = [
        Theme::Catppuccin(CatppuccinFlavor::Mocha),
        Theme::Catppuccin(CatppuccinFlavor::Macchiato),
        Theme::Catppuccin(CatppuccinFlavor::Frappe),
        Theme::Catppuccin(CatppuccinFlavor::Latte),
        Theme::RosePine(RosePineFlavor::Main),
        Theme::RosePine(RosePineFlavor::Moon),
        Theme::RosePine(RosePineFlavor::Dawn),
        Theme::TokyoNight(TokyoNightFlavor::Night),
        Theme::TokyoNight(TokyoNightFlavor::Storm),
        Theme::TokyoNight(TokyoNightFlavor::Day),
    ];
    for t in themes {
        assert_ne!(t.border_focused(), t.border_unfocused(), "theme {t:?}");
        assert_ne!(t.border_monocle(), t.border_unfocused(), "theme {t:?}");
    }
}

#[test]
fn resolve_color_handles_all_cases() {
    let t = Theme::Catppuccin(CatppuccinFlavor::Mocha);
    assert_eq!(t.resolve_color("", "#default"), "#default");
    assert_eq!(t.resolve_color("#ff0000", "#default"), "#ff0000");
    assert_eq!(t.resolve_color("blue", "#default"), "#89b4fa");
    assert_eq!(t.resolve_color("chartreuse", "#default"), "chartreuse");
}

#[test]
fn each_theme_has_distinct_base() {
    let bases: Vec<String> = [
        Theme::Catppuccin(CatppuccinFlavor::Mocha),
        Theme::Catppuccin(CatppuccinFlavor::Macchiato),
        Theme::Catppuccin(CatppuccinFlavor::Frappe),
        Theme::Catppuccin(CatppuccinFlavor::Latte),
        Theme::RosePine(RosePineFlavor::Main),
        Theme::RosePine(RosePineFlavor::Moon),
        Theme::RosePine(RosePineFlavor::Dawn),
        Theme::TokyoNight(TokyoNightFlavor::Night),
        Theme::TokyoNight(TokyoNightFlavor::Storm),
        Theme::TokyoNight(TokyoNightFlavor::Day),
    ]
    .iter()
    .map(|t| t.bar_colors().background.clone())
    .collect();
    for (i, a) in bases.iter().enumerate() {
        for (j, b) in bases.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "themes {i} and {j} share the same background");
            }
        }
    }
}

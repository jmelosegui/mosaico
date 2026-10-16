use crate::config::AnimationConfig;
use crate::motion::Easing;

use super::*;

/// The quoted names in the comment just above `easing =` in the
/// `[animations]` section of `toml_str`.
fn easing_names(toml_str: &str) -> Vec<String> {
    let section = toml_str
        .split("[animations]")
        .nth(1)
        .expect("no [animations]");
    let lines: Vec<&str> = section.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.starts_with("easing ="))
        .expect("no easing");
    let comment: Vec<&str> = lines[..at]
        .iter()
        .rev()
        .take_while(|l| l.starts_with('#'))
        .copied()
        .collect();
    let comment = comment.join("\n");
    comment
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

#[test]
fn config_template_writes_out_every_animation_setting_at_its_default() {
    // Arrange
    let toml_str = generate_config();

    // Act
    let file: toml::Table = toml::from_str(&toml_str).unwrap();
    let mut config: crate::Config = toml::from_str(&toml_str).unwrap();
    config.validate();

    // Assert -- the section is there, not left to the defaults.
    let section = file["animations"].as_table().unwrap();
    let mut keys: Vec<&str> = section.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["duration", "easing", "enabled", "follow_system"]);
    assert_eq!(config.animations, AnimationConfig::default());
    assert!(config.animations.enabled, "transitions are on by default");
}

#[test]
fn every_easing_the_config_template_names_is_one_the_config_takes() {
    // Arrange -- the curve is written with placeholders, so a curve with
    // numbers in their place stands for it.
    let curve = "cubic-bezier(x1, y1, x2, y2)";
    let presets = [
        "linear",
        "ease-out-quad",
        "ease-out-cubic",
        "ease-in-out-cubic",
        "ease-out-expo",
    ];

    // Act
    let names = easing_names(&generate_config());

    // Assert -- every preset and the curve are named, and every name the
    // template writes is one the config takes.
    for name in presets.iter().chain([&curve]) {
        assert!(names.iter().any(|n| n == name), "{name} missing: {names:?}");
    }
    for name in &names {
        let written = if name == curve {
            "cubic-bezier(0.2, 0, 0, 1)"
        } else {
            name
        };
        assert!(written.parse::<Easing>().is_ok(), "{name} is no easing");
    }
}

use mosaico_core::config::{BarChoice, ClickBinding};

use super::*;
use crate::commands::test_support::Harness;

/// What doctor says about the clicks of a `bar.toml` holding `contents`.
fn said(test: &str, contents: &str) -> Vec<String> {
    let h = Harness::new(test);
    std::fs::create_dir_all(h.config.dir()).unwrap();
    std::fs::write(h.config.bar_path(), contents).unwrap();
    let lines = bar_click_lines(&h.config);
    let _ = std::fs::remove_dir_all(h.config.dir());
    lines
}

/// The line that the bar has `count` bindings.
fn has(count: usize) -> String {
    format!("  {OK} The bar has {count} click binding(s)")
}

/// The line that the bar leaves out the entry with `keys`, for `why`.
fn left_out(keys: &str, why: &str) -> String {
    format!("  {FAIL} bar.toml: [[mouse.click]] with {keys} is ignored: {why}")
}

#[test]
fn the_default_bar_has_three_click_bindings() {
    // Act
    let lines = click_lines(&BarConfig::default());

    // Assert
    assert_eq!(lines, [has(3)]);
}

#[test]
fn the_count_is_of_the_bindings_the_bar_has_not_of_the_entries() {
    // Act -- an entry added, an empty list, and a default removed, which is
    // no mistake.
    let added = said(
        "doctor-bar-added",
        "[[mouse.click]]\nwidget = \"clock\"\nbutton = \"middle\"\naction = \"retile\"\n",
    );
    let empty = said("doctor-bar-empty", "[mouse]\nclick = []\n");
    let removed = said(
        "doctor-bar-removed",
        "[[mouse.click]]\nwidget = \"layout\"\nbutton = \"right\"\naction = \"none\"\n",
    );

    // Assert
    assert_eq!(added, [has(4)]);
    assert_eq!(empty, [has(3)]);
    assert_eq!(removed, [has(2)]);
}

#[test]
fn a_bar_that_takes_no_clicks_says_so() {
    // Act
    let mouse_off = said("doctor-bar-mouse-off", "[mouse]\nenabled = false\n");
    let bar_off = said("doctor-bar-off", "enabled = false\n");

    // Assert
    let no_clicks = |why: &str| format!("  {OK} The bar takes no clicks: {why}");
    assert_eq!(mouse_off, [no_clicks("[mouse] enabled = false")]);
    assert_eq!(bar_off, [no_clicks("it is off (enabled = false)")]);
}

#[test]
fn each_entry_the_bar_leaves_out_is_named_with_why() {
    // Arrange -- the second entry runs.
    let mut bar = BarConfig::default();
    bar.mouse.click = vec![
        ClickBinding::new("layout", "left", "close-focused"),
        ClickBinding::new("clock", "middle", "retile"),
        ClickBinding::new("radio", "left", "retile"),
        ClickBinding::new("date", "fourth", "retile"),
        ClickBinding::new("date", "left", "fly"),
    ];

    // Act
    let lines = click_lines(&bar);

    // Assert
    let entry = |widget: &str, button: &str, action: &str| {
        format!("widget = \"{widget}\", button = \"{button}\" and action = \"{action}\"")
    };
    let focused = "the action is about the focused window, which can be on another \
                   monitor than the bar; a click may only run an action on a monitor or \
                   the whole daemon";
    assert_eq!(
        lines,
        [
            has(4),
            left_out(&entry("layout", "left", "close-focused"), focused),
            left_out(&entry("radio", "left", "retile"), "no widget has that type"),
            left_out(
                &entry("date", "fourth", "retile"),
                "the button is not left, right or middle"
            ),
            left_out(&entry("date", "left", "fly"), "unknown action: fly"),
        ]
    );
}

#[test]
fn an_entry_with_a_key_left_out_is_named_with_the_keys_it_has() {
    // Act
    let lines = said(
        "doctor-bar-key",
        "[[mouse.click]]\nwidget = \"date\"\naction = \"retile\"\n",
    );

    // Assert
    let keys = "widget = \"date\" and action = \"retile\"";
    assert_eq!(lines, [has(3), left_out(keys, "it has no button")]);
}

#[test]
fn a_bar_toml_that_does_not_load_or_is_missing_gets_no_line_here() {
    // Arrange -- the file check reports a file that does not load.
    let h = Harness::new("doctor-bar-missing");

    // Act
    let invalid = said("doctor-bar-invalid", "height = = 40\n");
    let missing = bar_click_lines(&h.config);

    // Assert
    assert!(invalid.is_empty() && missing.is_empty());
}

/// What doctor says about the bar, with `config.toml` and `bar.toml`
/// holding `config` and `bar`.
fn doctor_says(test: &str, config: &str, bar: &str) -> String {
    let mut h = Harness::new(test);
    std::fs::create_dir_all(h.config.dir()).unwrap();
    std::fs::write(h.config.config_path(), config).unwrap();
    std::fs::write(h.config.bar_path(), bar).unwrap();
    h.run(crate::commands::doctor::execute);
    let _ = std::fs::remove_dir_all(h.config.dir());
    h.out().to_string()
}

#[test]
fn the_bar_in_use_is_named() {
    // Act
    let builtin = provider_lines(&ProviderCheck::Known(BarChoice::Mosaico));
    let none = provider_lines(&ProviderCheck::Known(BarChoice::Off));

    // Assert
    assert_eq!(builtin, [format!("  {OK} Bar: mosaico")]);
    assert_eq!(
        none,
        [
            format!("  {OK} Bar: none"),
            format!("  {OK} bar.toml is not read: the bar is \"none\", not mosaico's own"),
        ]
    );
}

#[test]
fn a_bar_mosaico_does_not_know_is_said_once_with_what_it_is_read_as() {
    // Act
    let out = doctor_says(
        "doctor-bar-unknown-provider",
        "[bar]\nprovider = \"elsewhere\"\n",
        "",
    );

    // Assert -- once, in the daemon's words, not again as a refused value.
    let warning = ProviderCheck::Unknown("elsewhere".into())
        .warning()
        .unwrap();
    assert!(out.contains(&format!("  {WARN} Bar: {warning}\n")), "{out}");
    assert_eq!(out.matches("elsewhere").count(), 1, "{out}");
}

#[test]
fn under_another_bar_bar_toml_is_not_checked() {
    // Act -- a bar.toml with a mistake doctor would otherwise name.
    let out = doctor_says(
        "doctor-bar-external",
        "[bar]\nprovider = \"External\"\n",
        "heigth = 30\n",
    );

    // Assert
    assert!(
        out.contains("bar.toml is not read: the bar is \"external\""),
        "{out}"
    );
    assert!(
        !out.contains("heigth") && !out.contains("bar.toml is valid"),
        "{out}"
    );
}

#[test]
fn a_built_in_bar_turned_off_is_pointed_to_the_provider() {
    // Act
    let out = doctor_says("doctor-bar-off-hint", "", "enabled = false\n");

    // Assert
    let hint = off_line(&BarConfig {
        enabled: false,
        ..BarConfig::default()
    })
    .unwrap();
    assert!(out.contains(&format!("{hint}\n")), "{out}");
    assert!(hint.contains("provider = \"none\""));
    assert_eq!(off_line(&BarConfig::default()), None);
}

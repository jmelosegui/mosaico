use super::*;
use crate::config::schema::{BAR_SETTINGS, CONFIG_SETTINGS};
use crate::config::template::{generate_bar, generate_config};

#[test]
fn the_files_mosaico_writes_have_no_problems() {
    assert_eq!(review(&generate_config(), CONFIG_SETTINGS), []);
    assert_eq!(review(&generate_bar(), BAR_SETTINGS), []);
}

#[test]
fn a_number_out_of_range_is_named_with_its_label_and_range() {
    // Act
    let problems = review("[layout]\ngap = 5000\n", CONFIG_SETTINGS);

    // Assert
    let text: Vec<String> = problems.iter().map(ToString::to_string).collect();
    assert_eq!(text, ["Gap (layout.gap) is 5000; it takes 0 to 200 px"]);
}

#[test]
fn a_whole_number_for_a_fraction_is_read_as_one() {
    // Act
    let problems = review("[layout]\nratio = 2\n", CONFIG_SETTINGS);

    // Assert
    let text: Vec<String> = problems.iter().map(ToString::to_string).collect();
    assert_eq!(
        text,
        ["Split ratio (layout.ratio) is 2; it takes 0.1 to 0.9"]
    );
    assert_eq!(review("[layout]\nratio = 0.5\n", CONFIG_SETTINGS), []);
}

#[test]
fn a_name_the_setting_does_not_list_is_refused() {
    // Act
    let problems = review("[layout]\nhiding = \"sideways\"\n", CONFIG_SETTINGS);

    // Assert
    assert!(
        matches!(&problems[..], [SettingProblem::Refused { setting, .. }] if setting.key == "layout.hiding"),
        "{problems:?}"
    );
}

#[test]
fn a_key_no_setting_has_is_unknown_and_lists_are_left_to_the_loader() {
    // Act
    let problems = review(
        "heigth = 30\nmonitors = [0]\n[[left]]\ntype = \"clock\"\n",
        BAR_SETTINGS,
    );

    // Assert
    let text: Vec<String> = problems.iter().map(ToString::to_string).collect();
    assert_eq!(text, ["heigth is not a setting mosaico knows"]);
}

#[test]
fn a_file_that_does_not_parse_has_no_problems_here() {
    assert_eq!(review("gap = = 1", CONFIG_SETTINGS), []);
}

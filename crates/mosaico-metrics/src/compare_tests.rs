use super::*;

fn baseline(entries: &[(&str, &str, u64)]) -> Values {
    let mut values = Values::new();
    for &(section, name, value) in entries {
        values
            .entry(section.to_owned())
            .or_default()
            .insert(name.to_owned(), value);
    }
    values
}

#[test]
fn a_report_that_matches_its_baseline_has_no_changes() {
    // Arrange
    let mut report = Report::new();
    report.set("open", "place", 2, Better::Lower);

    // Act
    let changes = compare(&baseline(&[("open", "place", 2)]), &report);

    // Assert
    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn more_work_is_worse_and_less_is_better() {
    // Arrange
    let mut report = Report::new();
    report.set("open", "place", 3, Better::Lower);
    report.set("open", "show_border", 1, Better::Lower);
    let before = baseline(&[("open", "place", 2), ("open", "show_border", 2)]);

    // Act
    let changes = compare(&before, &report);

    // Assert
    let verdicts: Vec<(&str, bool)> = changes
        .iter()
        .map(|c| (c.metric.as_str(), c.worse))
        .collect();
    assert_eq!(verdicts, [("place", true), ("show_border", false)]);
}

#[test]
fn fewer_tests_are_worse() {
    // Arrange
    let mut report = Report::new();
    report.set("core", "tests", 9, Better::Higher);

    // Act
    let changes = compare(&baseline(&[("core", "tests", 10)]), &report);

    // Assert
    assert_eq!(changes.len(), 1);
    assert!(changes[0].worse);
}

#[test]
fn a_metric_one_side_leaves_out_counts_as_zero() {
    // Arrange -- the report no longer focuses, and newly hides.
    let mut report = Report::new();
    report.set("switch", "hide", 2, Better::Lower);
    let before = baseline(&[("switch", "focus", 1)]);

    // Act
    let changes = compare(&before, &report);

    // Assert
    let shown: Vec<String> = changes.iter().map(ToString::to_string).collect();
    assert_eq!(
        shown,
        ["better switch.focus: 1 -> 0", "worse  switch.hide: 0 -> 2"]
    );
}

#[test]
fn a_new_section_is_a_change() {
    // Arrange
    let mut report = Report::new();
    report.set("close", "focus", 1, Better::Lower);

    // Act
    let changes = compare(&Values::new(), &report);

    // Assert
    assert_eq!(changes.len(), 1);
    assert_eq!((changes[0].from, changes[0].to), (0, 1));
}

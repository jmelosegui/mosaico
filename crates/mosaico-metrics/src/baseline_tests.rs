use std::fs;
use std::path::PathBuf;

use crate::Better;

use super::*;

const BLESS: &str = "bless it";

/// A baseline path in a directory of the test's own, removed when the
/// test ends.
struct TempBaseline {
    dir: PathBuf,
    path: PathBuf,
}

impl TempBaseline {
    fn new(test: &str) -> Self {
        let dir = std::env::temp_dir()
            .join("mosaico-metrics-tests")
            .join(format!("{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("metrics").join("baseline.toml");
        Self { dir, path }
    }

    fn baseline(&self) -> Baseline<'_> {
        Baseline::new(&self.path, BLESS)
    }
}

impl Drop for TempBaseline {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn report(place: u64) -> Report {
    let mut report = Report::new();
    report.set("open", "place", place, Better::Lower);
    report
}

#[test]
fn a_missing_baseline_fails_and_says_how_to_record_one() {
    // Arrange
    let temp = TempBaseline::new("missing");

    // Act
    let failure = temp.baseline().check(&report(2), false).unwrap_err();

    // Assert
    assert!(matches!(failure, Failure::Missing { .. }));
    assert!(failure.to_string().contains(BLESS), "{failure}");
}

#[test]
fn blessing_records_a_baseline_the_next_check_passes() {
    // Arrange
    let temp = TempBaseline::new("bless");

    // Act
    temp.baseline().check(&report(2), true).unwrap();
    let again = temp.baseline().check(&report(2), false);

    // Assert
    assert!(again.unwrap().is_empty());
    let text = fs::read_to_string(&temp.path).unwrap();
    assert!(
        text.starts_with(&format!("# Measured by `{BLESS}`")),
        "{text}"
    );
}

#[test]
fn loading_gives_back_what_was_recorded() {
    // Arrange
    let temp = TempBaseline::new("load");
    let missing = temp.baseline().load();
    temp.baseline().check(&report(2), true).unwrap();

    // Act
    let values = temp.baseline().load().unwrap();

    // Assert
    assert!(matches!(missing, Err(Failure::Missing { .. })));
    assert_eq!(values, report(2).values());
}

#[test]
fn a_change_fails_the_check_and_leaves_the_baseline_alone() {
    // Arrange
    let temp = TempBaseline::new("changed");
    temp.baseline().check(&report(2), true).unwrap();
    let before = fs::read_to_string(&temp.path).unwrap();

    // Act -- one fewer placement is better, but still has to be recorded.
    let failure = temp.baseline().check(&report(1), false).unwrap_err();

    // Assert
    let Failure::Changed { changes, .. } = &failure else {
        panic!("expected a change, got {failure}");
    };
    assert_eq!(changes.len(), 1);
    assert!(!changes[0].worse);
    assert_eq!(fs::read_to_string(&temp.path).unwrap(), before);
}

#[test]
fn blessing_a_change_replaces_the_baseline() {
    // Arrange
    let temp = TempBaseline::new("rebless");
    temp.baseline().check(&report(2), true).unwrap();

    // Act
    let changes = temp.baseline().check(&report(3), true).unwrap();

    // Assert
    assert_eq!(changes.len(), 1);
    assert!(temp.baseline().check(&report(3), false).unwrap().is_empty());
}

#[test]
fn a_baseline_that_is_not_toml_is_unusable() {
    // Arrange
    let temp = TempBaseline::new("garbage");
    fs::create_dir_all(temp.path.parent().unwrap()).unwrap();
    fs::write(&temp.path, "place = [").unwrap();

    // Act
    let failure = temp.baseline().check(&report(2), false).unwrap_err();

    // Assert
    assert!(matches!(failure, Failure::Unusable { .. }), "{failure}");
}

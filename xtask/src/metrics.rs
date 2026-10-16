//! `cargo xtask metrics [--bless]`: checks the committed metrics
//! baselines, or records new ones, and shows them.
//!
//! Two baselines are checked. `metrics/code.toml` holds code health
//! counts per crate, measured here. `metrics/tiler.toml` holds what the
//! tiling manager asks of the platform in common situations, measured by
//! a unit test in mosaico-core that this runs.

use std::path::Path;

use mosaico_metrics::{Baseline, Better, Report, Values, compare};

use crate::{cargo, code, source, summary};

const BLESS_WITH: &str = "cargo xtask metrics --bless";

/// The code health columns, in the order the table shows them.
const CODE_COLUMNS: &[&str] = &[
    "tests",
    "files_over_line_limit",
    "lines_over_line_limit",
    "unsafe_keywords",
    "unwraps_outside_tests",
    "os_names",
];

/// The tiler columns the table shows. `metrics/tiler.toml` has every
/// kind of call.
const TILER_COLUMNS: &[&str] = &[
    "calls",
    "place",
    "show_border",
    "border_renders",
    "move_cursor",
    "focus",
    "hide",
    "show",
    "repaint",
];

/// Checks both baselines, or records them when `bless` is set, and shows
/// them. Returns whether they pass.
pub fn run(root: &Path, bless: bool) -> bool {
    let code = check_code(root, bless);
    let tiler = check_tiler(root, bless);
    let (Some(code), Some(tiler)) = (code, tiler) else {
        return false;
    };
    println!("\nCode health, metrics/code.toml (more tests is better, fewer of the rest)\n");
    print!("{}", summary::table(&code, "crate", CODE_COLUMNS, "-"));
    println!(
        "\nPlatform calls per scenario after coalescing, metrics/tiler.toml (fewer is better)\n"
    );
    print!("{}", summary::table(&tiler, "scenario", TILER_COLUMNS, "0"));
    let verb = if bless { "recorded" } else { "match" };
    println!("\nAll counts {verb} the baselines.");
    true
}

/// Measures the code health counts and checks them against their
/// baseline, returning them when they pass.
fn check_code(root: &Path, bless: bool) -> Option<Values> {
    let files = source::read_crates(root)
        .map_err(|e| eprintln!("could not read the crate sources: {e}"))
        .ok()?;
    let report = code::report(&files);
    let path = root.join("metrics").join("code.toml");
    match Baseline::new(&path, BLESS_WITH).check(&report, bless) {
        Ok(changes) => {
            for change in &changes {
                println!("recorded {change}");
            }
            Some(report.values())
        }
        Err(failure) => {
            eprintln!("{failure}");
            None
        }
    }
}

/// Runs the tiler metrics test, which checks or records its own
/// baseline, and returns the baseline when it passes. Shows the test's
/// output only when it fails.
fn check_tiler(root: &Path, bless: bool) -> Option<Values> {
    let path = root.join("metrics").join("tiler.toml");
    let baseline = Baseline::new(&path, BLESS_WITH);
    let before = baseline.load().unwrap_or_default();
    let mut test = cargo(root);
    test.args([
        "test",
        "--quiet",
        "-p",
        "mosaico-core",
        "--lib",
        "tiler::tests::metrics_tests",
    ]);
    if bless {
        test.env("MOSAICO_BLESS", "1");
    }
    let output = test
        .output()
        .map_err(|e| eprintln!("could not run the tiler metrics test: {e}"))
        .ok()?;
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        return None;
    }
    let after = baseline.load().map_err(|e| eprintln!("{e}")).ok()?;
    // Every tiler count is work done, so fewer is better.
    let mut measured = Report::new();
    for (section, metrics) in &after {
        for (name, &value) in metrics {
            measured.set(section, name, value, Better::Lower);
        }
    }
    for change in compare(&before, &measured) {
        println!("recorded {change}");
    }
    Some(after)
}

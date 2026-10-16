//! The work the tiling manager asks of the platform must not grow
//! unnoticed: each scenario's counts are checked against the committed
//! baseline in `metrics/tiler.toml`.

use std::path::Path;

use mosaico_metrics::{Baseline, Better, Report};

use super::fake_platform::Call;
use super::metrics_scenarios::SCENARIOS;
use super::metrics_support::Measured;

/// What records a new baseline, for failure messages to suggest.
const BLESS_WITH: &str = "cargo xtask metrics --bless";

/// The name a call is counted under.
fn kind(call: &Call) -> &'static str {
    match call {
        Call::Place(..) => "place",
        Call::PrePlace(..) => "pre_place",
        Call::PlaceFrames(..) => "place_frames",
        Call::SettleResponding => "settle_responding",
        Call::Repaint(..) => "repaint",
        Call::Focus(..) => "focus",
        Call::PutBehind(..) => "put_behind",
        Call::Close(..) => "close",
        Call::Minimize(..) => "minimize",
        Call::Hide(..) => "hide",
        Call::Show(..) => "show",
        Call::Reveal(..) => "reveal",
        Call::MoveCursor(..) => "move_cursor",
        Call::ShowBorder(..) => "show_border",
        Call::HideBorder(..) => "hide_border",
        Call::DropBorder(..) => "drop_border",
        Call::SetCorners(..) => "set_corners",
        Call::ResetCorners(..) => "reset_corners",
    }
}

/// Every scenario's calls by kind, with the total and the borders drawn
/// from scratch. Kinds a scenario never asks for are left out.
fn measure() -> Report {
    let mut report = Report::new();
    for &(name, run) in SCENARIOS {
        let Measured { calls, renders } = run();
        report.set(name, "calls", calls.len() as u64, Better::Lower);
        report.set(name, "border_renders", renders as u64, Better::Lower);
        for call in &calls {
            let count = report.get(name, kind(call)).map_or(0, |m| m.value);
            report.set(name, kind(call), count + 1, Better::Lower);
        }
    }
    report
}

#[test]
fn the_work_asked_of_the_platform_matches_the_baseline() {
    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2);
    let path = root
        .expect("the crate is in crates/")
        .join("metrics")
        .join("tiler.toml");
    let bless = std::env::var_os("MOSAICO_BLESS").is_some();

    // Act
    let report = measure();

    // Assert
    match Baseline::new(&path, BLESS_WITH).check(&report, bless) {
        Ok(changes) => {
            for change in changes {
                eprintln!("recorded {change}");
            }
        }
        Err(failure) => panic!("{failure}"),
    }
}

#[test]
fn every_scenario_counts_the_same_each_run() {
    // Act
    let first = measure();
    let second = measure();

    // Assert -- a count that wobbles would fail the baseline at random.
    assert_eq!(first, second);
}

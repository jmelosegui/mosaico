//! `cargo xtask`: development tasks for Mosaico.
//!
//! - `metrics [--bless]` checks the committed metrics baselines, or
//!   records new ones. See [`xtask::metrics`].
//! - `size` builds the release binaries, checks `mosaico` against its size
//!   ceiling and reports the settings app's size. See [`xtask::size`].

use std::env;
use std::path::Path;
use std::process::ExitCode;

use xtask::{metrics, size};

const USAGE: &str = "usage: cargo xtask metrics [--bless] | cargo xtask size";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace root");
    let passed = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["metrics"] => metrics::run(root, false),
        ["metrics", "--bless"] => metrics::run(root, true),
        ["size"] => size::run(root),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    if passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

//! Development tasks for Mosaico.

use std::env;
use std::path::Path;
use std::process::Command;

pub mod code;
pub mod metrics;
mod scan;
pub mod size;
pub mod source;
pub mod summary;

/// The cargo that runs this task, set to run in the workspace `root`.
fn cargo(root: &Path) -> Command {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut command = Command::new(cargo);
    command.current_dir(root);
    command
}

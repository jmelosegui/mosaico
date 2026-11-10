//! `mosaico run`: runs one action as a click on a bar runs it, for bars
//! that run a command on a click.

use clap::Args;
use mosaico_core::api::client::{self, MonitorRef};

use super::bar_api::{BarIo, finish};
use super::context::Exit;

/// What to run, and where.
#[derive(Debug, Args)]
pub struct RunArgs {
    /// The action, as `bar.toml`'s click bindings name it, such as
    /// `goto-workspace-3`
    pub action: String,
    /// The monitor it acts on, by id or by index from the left; the
    /// focused one when not given
    #[arg(long, value_name = "ID")]
    pub monitor: Option<MonitorRef>,
}

/// Runs the action and waits for the daemon to answer.
pub fn execute(io: &mut BarIo, args: RunArgs) -> Exit {
    let monitor = args.monitor.as_ref();
    let outcome = client::run(io.api, "mosaico run", &args.action, monitor);
    finish(outcome, io.err)
}

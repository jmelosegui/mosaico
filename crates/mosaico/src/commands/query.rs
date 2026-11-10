//! `mosaico query`: prints the state bars are shown, once.

use clap::Args;
use mosaico_core::api::client::{self, FieldPath, MonitorRef, Query};

use super::bar_api::{BarIo, finish};
use super::context::Exit;

/// What to print.
#[derive(Debug, Args)]
pub struct QueryArgs {
    /// The topics to print (daemon, theme, monitors, focus); all when none
    pub topics: Vec<String>,
    /// Print only this monitor, by id or by index from the left
    #[arg(long, value_name = "ID")]
    pub monitor: Option<MonitorRef>,
    /// Print only this value, as text, such as `focus.window.title`
    #[arg(long, value_name = "PATH")]
    pub field: Option<FieldPath>,
}

/// Prints the state as JSON, or one value of it as text.
pub fn execute(io: &mut BarIo, args: QueryArgs) -> Exit {
    let query = Query {
        topics: (!args.topics.is_empty()).then_some(args.topics),
        monitor: args.monitor,
        field: args.field,
    };
    let outcome = client::query(io.api, "mosaico query", &query, io.out);
    finish(outcome, io.err)
}

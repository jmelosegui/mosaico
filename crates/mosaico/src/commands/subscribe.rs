//! `mosaico subscribe`: follows the state bars are shown, a line for
//! each change, for bars that read a command's output.

use clap::Args;
use mosaico_core::api::client::{self, FieldPath, MonitorRef, Output, Streams, Subscription};

use super::bar_api::{BarIo, finish};
use super::context::Exit;

/// What to follow and print.
#[derive(Debug, Args)]
pub struct SubscribeArgs {
    /// The bar this command speaks for, such as `yasb`
    #[arg(long, value_name = "NAME")]
    pub bar: Option<String>,
    /// The topics to follow, comma separated; all when not given
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub topics: Option<Vec<String>>,
    /// Show only this monitor, by id or by index from the left, and run
    /// requests that name none on it
    #[arg(long, value_name = "ID")]
    pub monitor: Option<MonitorRef>,
    /// Print the whole state on subscribing and on every change
    #[arg(long, conflicts_with = "field")]
    pub full: bool,
    /// Print only this value, as text, each time it changes
    #[arg(long, value_name = "PATH")]
    pub field: Option<FieldPath>,
    /// Wait for mosaico when it is not running, and connect again when it
    /// restarts
    #[arg(long)]
    pub wait: bool,
    /// Send the requests read on standard input, one JSON object a line;
    /// ends when none comes for 15 seconds
    #[arg(long)]
    pub requests: bool,
}

/// Prints the session's messages until it, the input, the output or the
/// process that started this one ends.
pub fn execute(io: &mut BarIo, args: SubscribeArgs) -> Exit {
    let output = match (args.full, args.field) {
        (_, Some(field)) => Output::Field(field),
        (true, None) => Output::Full,
        (false, None) => Output::Messages,
    };
    let subscription = Subscription {
        bar: args.bar,
        topics: args.topics,
        monitor: args.monitor,
        output,
        wait: args.wait,
        requests: args.requests,
        ..Subscription::default()
    };
    let streams = Streams {
        input: (io.input)(),
        out: &mut *io.out,
        parent: (io.parent)(),
    };
    let outcome = client::subscribe(io.api, "mosaico subscribe", &subscription, streams);
    finish(outcome, io.err)
}

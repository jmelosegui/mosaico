//! `mosaico script run`: runs a script until it ends, waiting for the
//! daemon whenever it is not there.

use std::time::Duration;

use mosaico_script::{Options, Outcome, Script};

use super::paths::{Folders, name_of};
use crate::commands::context::{Context, Exit, complain, say};

/// How often a script waiting for the daemon checks whether it is back.
pub const DAEMON_POLL: Duration = Duration::from_secs(1);

/// Runs the script `target` names with `args`, each `key=value`.
///
/// A script that handles events and finds no daemon, or whose daemon
/// stops, waits for it to start and goes on with the same handlers, so
/// a script started at logon survives the daemon restarting.
pub fn execute(cx: &mut Context, target: &str, args: &[String], fail_fast: bool) -> Exit {
    let folders = Folders::new(cx.config.dir());
    let path = folders.resolve(target);
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(e) => {
            complain!(cx, "Error: could not read {}: {e}", path.display());
            if !path.exists() {
                complain!(cx, "List the scripts with `mosaico script list`.");
            }
            return Exit::Failure;
        }
    };
    let args = match parse_args(args) {
        Ok(args) => args,
        Err(message) => {
            complain!(cx, "Error: {message}");
            return Exit::Failure;
        }
    };
    let options = Options {
        name: name_of(&path),
        lib: Some(folders.lib),
        args,
        fail_fast,
    };
    let mut script = match Script::new(source, (cx.scripts)(), options) {
        Ok(script) => script,
        Err(e) => {
            complain!(cx, "Error: {e}");
            return Exit::Failure;
        }
    };
    let mut waiting = false;
    loop {
        match script.run() {
            Outcome::Finished | Outcome::Stopped(0) => return Exit::Success,
            Outcome::Stopped(_) => return Exit::Failure,
            Outcome::Failed(message) => {
                complain!(cx, "Error: {message}");
                return Exit::Failure;
            }
            Outcome::DaemonGone => {
                if !waiting {
                    say!(cx, "Waiting for mosaico to start...");
                }
                while !cx.daemon.is_running() {
                    cx.clock.sleep(DAEMON_POLL);
                }
                waiting = true;
            }
        }
    }
}

/// `key=value` arguments as pairs.
fn parse_args(args: &[String]) -> Result<Vec<(String, String)>, String> {
    args.iter()
        .map(|arg| match arg.split_once('=') {
            Some((key, value)) if !key.is_empty() => Ok((key.to_string(), value.to_string())),
            _ => Err(format!("`{arg}` is not a key=value argument")),
        })
        .collect()
}

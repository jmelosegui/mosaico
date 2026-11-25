//! `mosaico script run`: runs a script until it ends, waiting for the
//! daemon whenever it is not there, or starts it in the background.

use std::cell::RefCell;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use mosaico_script::{Options, Outcome, Script};

use super::paths::{Folders, name_of};
use crate::commands::context::{Context, Exit, complain, say};

/// How often a script waiting for the daemon checks whether it is back.
pub const DAEMON_POLL: Duration = Duration::from_secs(1);

/// What `mosaico script run` was asked.
pub struct Run<'a> {
    /// The script's name, or the path of its file.
    pub target: &'a str,
    /// Its `key=value` arguments.
    pub args: &'a [String],
    /// Stop at the first error in a handler or timer.
    pub fail_fast: bool,
    /// Start it detached, without a console, and return.
    pub background: bool,
    /// Write what it prints to this file instead of the console.
    pub log: Option<&'a Path>,
}

/// Runs the script `run` names.
///
/// A script that handles events and finds no daemon, or whose daemon
/// stops, waits for it to start and goes on with the same handlers, so
/// a script started at logon survives the daemon restarting.
pub fn execute(cx: &mut Context, run: &Run) -> Exit {
    let folders = Folders::new(cx.config.dir());
    let path = folders.resolve(run.target);
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(e) => {
            complain!(cx, "Error: could not read {}: {e}", path.display());
            if !path.exists() {
                complain!(cx, "List the scripts with `mosaico script list`.");
            }
            return Exit::Failure;
        }
    };
    let args = match parse_args(run.args) {
        Ok(args) => args,
        Err(message) => {
            complain!(cx, "Error: {message}");
            return Exit::Failure;
        }
    };
    let name = name_of(&path);
    if run.background {
        return in_background(cx, run, &name);
    }
    let mut env = (cx.scripts)();
    if let Some(log) = run.log {
        match open_log(log) {
            Ok(file) => {
                let file: mosaico_script::Output = Rc::new(RefCell::new(file));
                env.out = file.clone();
                env.err = file;
            }
            Err(e) => {
                complain!(cx, "Error: could not write {}: {e}", log.display());
                return Exit::Failure;
            }
        }
    }
    let err = env.err.clone();
    let options = Options {
        name,
        lib: Some(folders.lib),
        args,
        fail_fast: run.fail_fast,
    };
    let mut script = match Script::new(source, env, options) {
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
                if run.log.is_some() {
                    let _ = writeln!(err.borrow_mut(), "Error: {message}");
                }
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

/// Starts this command again, detached and without a console, writing
/// to the script's log, and returns at once.
fn in_background(cx: &mut Context, run: &Run, name: &str) -> Exit {
    let log = log_path(cx.config.dir(), name);
    let mut args = vec!["script".to_string(), "run".into(), run.target.into()];
    args.extend(run.args.iter().cloned());
    if run.fail_fast {
        args.push("--fail-fast".into());
    }
    args.push("--log".into());
    args.push(log.to_string_lossy().into_owned());
    match cx.processes.launch_self(&args) {
        Ok(_) => {
            say!(
                cx,
                "Started {name} in the background. It writes to {}",
                log.display()
            );
            Exit::Success
        }
        Err(e) => {
            complain!(cx, "Error: could not start {name}: {e}");
            Exit::Failure
        }
    }
}

/// Where the script called `name` writes when it runs in the background:
/// `logs\scripts\<name>.log` in the config folder, beside mosaico's log.
pub fn log_path(config: &Path, name: &str) -> PathBuf {
    config
        .join("logs")
        .join("scripts")
        .join(format!("{name}.log"))
}

fn open_log(path: &Path) -> std::io::Result<fs::File> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    OpenOptions::new().create(true).append(true).open(path)
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

//! What `mosaico doctor` says about a bar another program draws that
//! `[bar] provider` names: whether the program runs, and whether it
//! holds a session with mosaico.

use mosaico_core::api::{BarSession, BarStatus};
use mosaico_core::config::BarProgram;

use super::context::{Context, say};

const OK: &str = "\x1b[32m[ok]\x1b[0m";
const WARN: &str = "\x1b[33m[warn]\x1b[0m";

/// Says whether the program drawing the configured bar runs, and the
/// sessions it holds with mosaico in the daemon's `bar` status. Says nothing when the
/// configured bar is not one mosaico knows by name.
pub fn check_program(cx: &mut Context, bar: Option<&BarStatus>) {
    let Some(program) = cx.config.bar_provider().choice().program() else {
        return;
    };
    let pids = cx.processes.running(program.program);
    for line in program_lines(program, &pids, bar) {
        say!(cx, "{line}");
    }
}

/// What doctor says about `program`, running as `pids`, while the
/// daemon serves `bar`, if it answered.
fn program_lines(program: &BarProgram, pids: &[u32], bar: Option<&BarStatus>) -> Vec<String> {
    let title = program.title;
    let setup = format!("`mosaico bar setup {}`", program.bar);
    let processes = match pids {
        [] => {
            let line = format!("  {WARN} {title} is not running: start it, then run {setup}");
            return vec![line];
        }
        [pid] => format!("process {pid}"),
        _ => format!("processes {}", joined(pids)),
    };
    let mut lines = vec![format!("  {OK} {title} is running ({processes})")];
    let Some(bar) = bar else {
        return lines;
    };
    let sessions: Vec<u64> = (bar.sessions.iter())
        .filter(|s| !s.ended && speaks_for(s, program))
        .map(|s| s.session)
        .collect();
    // Widgets that run `mosaico query` open no session, so one missing is
    // not a fault.
    match sessions.as_slice() {
        [] => {}
        [one] => lines.push(format!(
            "  {OK} {title} is connected to mosaico (session {one})"
        )),
        _ => lines.push(format!(
            "  {OK} {title} is connected to mosaico (sessions {})",
            joined(&sessions)
        )),
    }
    lines
}

/// Whether `session` named the bar `program` draws in its hello.
fn speaks_for(session: &BarSession, program: &BarProgram) -> bool {
    (session.bar.as_deref()).is_some_and(|bar| bar.eq_ignore_ascii_case(program.bar))
}

/// `numbers` as "1, 2, 3".
fn joined<T: ToString>(numbers: &[T]) -> String {
    let each: Vec<String> = numbers.iter().map(T::to_string).collect();
    each.join(", ")
}

#[cfg(test)]
#[path = "tests/doctor_bar_program_tests.rs"]
mod tests;

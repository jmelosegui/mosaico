//! What `mosaico doctor` says about the bar API as the running daemon
//! serves it: the bar it runs, and each session with the space it keeps
//! clear of tiles.

use mosaico_core::api::{BarSession, BarStatus, MonitorTarget, Reserve};
use mosaico_core::ipc::{Command, DaemonStatus};

use super::context::{Context, say};

const OK: &str = "\x1b[32m[ok]\x1b[0m";
const WARN: &str = "\x1b[33m[warn]\x1b[0m";

/// Asks the running daemon for its bar API and says what it serves. Says
/// nothing when no daemon answers, which the daemon check reports, or
/// when the daemon is older than the bar API.
pub fn check_sessions(cx: &mut Context) {
    if !cx.daemon.is_running() {
        return;
    }
    let Ok(response) = cx.daemon.send(&Command::Status) else {
        return;
    };
    let Some(bar) = response
        .data::<DaemonStatus>()
        .and_then(|status| status.bar)
    else {
        return;
    };
    let configured = cx.config.bar_provider().choice().name().to_string();
    for line in session_lines(&bar, &configured) {
        say!(cx, "{line}");
    }
}

/// What doctor says about `bar`, served while `config.toml` picks the
/// provider named `configured`.
fn session_lines(bar: &BarStatus, configured: &str) -> Vec<String> {
    let mut lines = Vec::new();
    if bar.provider != configured {
        lines.push(format!(
            "  {WARN} The daemon runs the bar \"{}\", not \"{configured}\" as config.toml says: \
             it takes the file in once it reads without errors",
            bar.provider
        ));
    }
    if bar.sessions.is_empty() {
        lines.push(format!("  {OK} Bar API: no session open"));
        return lines;
    }
    let header = match bar.sessions.iter().filter(|s| !s.ended).count() {
        0 => "no session open".to_string(),
        1 => "1 session open".to_string(),
        open => format!("{open} sessions open"),
    };
    lines.push(format!("  {OK} Bar API: {header}"));
    lines.extend((bar.sessions.iter()).map(|session| session_line(session, &bar.provider)));
    lines
}

/// One session, who opened it and the space it keeps, under `provider`.
fn session_line(session: &BarSession, provider: &str) -> String {
    let who = match (&session.bar, &session.client) {
        (Some(bar), Some(client)) => format!("bar \"{bar}\" ({client})"),
        (Some(bar), None) => format!("bar \"{bar}\""),
        (None, Some(client)) => client.clone(),
        (None, None) => "a client that gave no name".to_string(),
    };
    let process = (session.process).map_or(String::new(), |p| format!(", process {p}"));
    let space = described(&session.reserves);
    let does = if session.ended {
        format!("ended; its space is kept a few seconds for a bar that starts again: {space}")
    } else if !session.reserves.is_empty() {
        format!("reserves {space}")
    } else if session.may_reserve {
        "reserves nothing".to_string()
    } else {
        format!("reserves nothing, and may not while the bar is \"{provider}\"")
    };
    format!("       Session {}: {who}{process}: {does}", session.session)
}

/// `reserves` as a list such as "top 32 on every monitor; left 8 on
/// monitor DEL-1".
fn described(reserves: &[Reserve]) -> String {
    let parts: Vec<String> = (reserves.iter())
        .map(|reserve| {
            let edges = [
                ("top", reserve.top),
                ("bottom", reserve.bottom),
                ("left", reserve.left),
                ("right", reserve.right),
            ];
            let kept: Vec<String> = (edges.iter())
                .filter(|(_, px)| *px > 0)
                .map(|(side, px)| format!("{side} {px}"))
                .collect();
            let monitor = match &reserve.monitor {
                MonitorTarget::All => "every monitor".to_string(),
                MonitorTarget::One(id) => format!("monitor {}", id.as_str()),
            };
            format!("{} on {monitor}", kept.join(", "))
        })
        .collect();
    parts.join("; ")
}

#[cfg(test)]
#[path = "tests/doctor_bar_sessions_tests.rs"]
mod tests;

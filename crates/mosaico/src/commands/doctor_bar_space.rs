//! What `mosaico doctor` says about the space tiles keep clear of at the
//! edges of each monitor: what the system keeps, what the bar keeps, and
//! the `[bar.reserve]` entries that keep nothing, and why.

use mosaico_core::bar::{ReservedSpace, unmatched_monitors};
use mosaico_core::config::{BarChoice, BarReserve};
use mosaico_core::tiler::{Edges, MonitorPlace, Side};

use super::context::{Context, say};

const OK: &str = "\x1b[32m[ok]\x1b[0m";
const WARN: &str = "\x1b[33m[warn]\x1b[0m";

/// Says, for each monitor as a daemon starting now would number it, what
/// space tiles keep clear of under the bar `choice`, and flags the
/// `[bar.reserve]` entries that keep nothing. Monitors that cannot be
/// listed are reported by the monitor check, so they get no line here.
pub fn check_space(cx: &mut Context, choice: BarChoice) {
    let Ok(displays) = cx.desktop.monitors() else {
        return;
    };
    let reserve = cx.config.bar_section().reserve;
    for line in space_lines(choice, &reserve, &MonitorPlace::list(&displays)) {
        say!(cx, "{line}");
    }
}

/// What doctor says about the space kept on `places` under the bar
/// `choice` with `reserve` in `[bar.reserve]`.
fn space_lines(choice: BarChoice, reserve: &BarReserve, places: &[MonitorPlace]) -> Vec<String> {
    let mut lines = vec![format!("  {OK} Space tiles keep clear of:")];
    let mut warnings = Vec::new();
    for place in places {
        let system = Edges::kept_by(&place.bounds, &place.work_area);
        let mut parts = vec![format!("the system keeps {}", described(system))];
        match choice {
            BarChoice::Mosaico => parts.push("mosaico's bar keeps the space it draws in".into()),
            BarChoice::External => {
                let space = ReservedSpace::on(reserve, place);
                parts.push(format!("[bar.reserve] keeps {}", described(space.edges)));
                warnings.extend(space.refused.iter().map(|side| refused(*side, place.index)));
            }
            BarChoice::Off => {}
        }
        let id = &place.id;
        lines.push(format!(
            "       On monitor {} ({id}): {}",
            place.index,
            parts.join("; ")
        ));
    }
    if choice != BarChoice::External {
        if *reserve != BarReserve::default() {
            let name = choice.name();
            lines.push(format!(
                "  {OK} [bar.reserve] is not used: the bar is \"{name}\", not \"external\""
            ));
        }
        return lines;
    }
    for name in unmatched_monitors(reserve, places) {
        warnings.push(format!(
            "  {WARN} [bar.reserve.monitor.{name}] keeps nothing: no connected monitor has \
             that id or position"
        ));
    }
    lines.extend(warnings);
    lines
}

/// The warning that `[bar.reserve]` keeps nothing at `side` of the monitor
/// at `index`.
fn refused(side: Side, index: usize) -> String {
    let side = side.name();
    format!(
        "  {WARN} [bar.reserve] keeps nothing at the {side} of monitor {index}: it would take \
         more than a quarter of the monitor"
    )
}

/// `edges` as a list of the edges that keep anything, such as "top 40,
/// left 8", or "nothing".
fn described(edges: Edges) -> String {
    let kept: Vec<String> = (Side::ALL.into_iter())
        .filter(|side| edges.get(*side) > 0)
        .map(|side| format!("{} {}", side.name(), edges.get(side)))
        .collect();
    if kept.is_empty() {
        return "nothing".into();
    }
    kept.join(", ")
}

#[cfg(test)]
#[path = "tests/doctor_bar_space_tests.rs"]
mod tests;

//! Which workspace the monitor of the bar the tests click shows, as the
//! daemon's `debug state` has it.

use std::thread;
use std::time::{Duration, Instant};

use super::common::TestInstance;
use super::strip::Bar;

/// How long the daemon is given to show a workspace.
const SWITCHED: Duration = Duration::from_secs(5);

/// The workspace the bar's monitor shows, once it shows `workspace` or
/// once the daemon had long enough to.
pub fn shown(instance: &TestInstance, bar: &Bar, workspace: usize) -> Option<usize> {
    let deadline = Instant::now() + SWITCHED;
    loop {
        let state = instance.stdout(&["debug", "state"]);
        let mut monitors = state.lines().filter(|line| line.starts_with("Monitor "));
        let shows = monitors.find_map(|line| showing(line, bar));
        if shows == Some(workspace) || Instant::now() >= deadline {
            return shows;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

/// The workspace a monitor's line of `debug state` says it shows, when
/// it is the line of the bar's monitor: the one whose tiling area starts
/// at the bar's bottom left corner.
fn showing(line: &str, bar: &Bar) -> Option<usize> {
    let area = line.split("work_area=").nth(1)?.split(',').next()?;
    let mut corner = area.split('+').skip(1).map(str::parse::<i32>);
    let at = (corner.next()?.ok()?, corner.next()?.ok()?);
    let after = line.split("active_ws=").nth(1)?;
    let number: String = after.chars().take_while(char::is_ascii_digit).collect();
    (at == (bar.left, bar.bottom)).then_some(number.parse().ok()?)
}

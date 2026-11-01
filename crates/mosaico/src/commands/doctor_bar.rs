//! What `mosaico doctor` says about `bar.toml`: whether it loads, the
//! bindings of clicks on the bar's widgets, and the `[[mouse.click]]`
//! entries the bar leaves out, and why.

use mosaico_core::bar::WidgetClicks;
use mosaico_core::config::schema::BAR_SETTINGS;
use mosaico_core::config::{BarConfig, ConfigStore};

use super::context::{Context, say};
use super::doctor::toml_file_line;

const OK: &str = "\x1b[32m[ok]\x1b[0m";
const FAIL: &str = "\x1b[31m[fail]\x1b[0m";

/// Says whether `bar.toml` loads, then, right after, how many bindings
/// clicks on the bar's widgets have, and flags the `[[mouse.click]]`
/// entries the bar leaves out. The bindings are the ones core resolves for
/// the bar, so this agrees with the daemon, which only logs the entries.
pub fn check_bar(cx: &mut Context) {
    let store = cx.config.clone();
    let loads = || store.try_load_bar().map(|_| ());
    let file = toml_file_line("bar.toml", &store.bar_path(), loads);
    say!(cx, "{file}");
    super::doctor_settings::check_settings(cx, "bar.toml", &store.bar_path(), BAR_SETTINGS);
    for line in bar_click_lines(&store) {
        say!(cx, "{line}");
    }
}

/// What doctor says about the clicks of the `bar.toml` in `store`. A
/// file that is missing has the defaults, which all run, and one that does
/// not load is reported by the file check, so neither gets a line.
fn bar_click_lines(store: &ConfigStore) -> Vec<String> {
    match store.try_load_bar() {
        Ok(bar) => click_lines(&bar),
        Err(_) => Vec::new(),
    }
}

/// What doctor says about the clicks of `bar`: how many bindings they
/// have, or that the bar takes none, then each entry the bar leaves out,
/// and why.
fn click_lines(bar: &BarConfig) -> Vec<String> {
    let has = if !bar.enabled {
        format!("  {OK} The bar takes no clicks: it is off (enabled = false)")
    } else if !bar.mouse.enabled {
        format!("  {OK} The bar takes no clicks: [mouse] enabled = false")
    } else {
        let count = WidgetClicks::new(&bar.mouse).len();
        format!("  {OK} The bar has {count} click binding(s)")
    };
    let rejected = WidgetClicks::rejected(&bar.mouse);
    let left_out = rejected.iter().map(|r| format!("  {FAIL} bar.toml: {r}"));
    std::iter::once(has).chain(left_out).collect()
}

#[cfg(test)]
#[path = "tests/doctor_bar_tests.rs"]
mod tests;

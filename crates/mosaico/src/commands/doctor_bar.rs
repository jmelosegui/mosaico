//! What `mosaico doctor` says about the bar: which one shows mosaico,
//! then, for the built-in one, whether `bar.toml` loads, the bindings of
//! clicks on the bar's widgets, and the `[[mouse.click]]` entries the bar
//! leaves out, and why.

use mosaico_core::bar::WidgetClicks;
use mosaico_core::config::schema::BAR_SETTINGS;
use mosaico_core::config::{BarConfig, ConfigStore, ProviderCheck};

use super::context::{Context, say};
use super::doctor::toml_file_line;

const OK: &str = "\x1b[32m[ok]\x1b[0m";
const WARN: &str = "\x1b[33m[warn]\x1b[0m";
const FAIL: &str = "\x1b[31m[fail]\x1b[0m";

/// Says which bar shows mosaico, as the daemon reads `[bar] provider`.
/// For the built-in bar it then says whether `bar.toml` loads, how many
/// bindings clicks on the bar's widgets have, and flags the
/// `[[mouse.click]]` entries the bar leaves out. The bindings are the ones
/// core resolves for the bar, so this agrees with the daemon, which only
/// logs the entries. For any other bar, `bar.toml` is not read.
pub fn check_bar(cx: &mut Context) {
    let store = cx.config.clone();
    let provider = store.bar_provider();
    for line in provider_lines(&provider) {
        say!(cx, "{line}");
    }
    if !provider.choice().is_builtin() {
        return;
    }
    let loads = || store.try_load_bar().map(|_| ());
    let file = toml_file_line("bar.toml", &store.bar_path(), loads);
    say!(cx, "{file}");
    super::doctor_settings::check_settings(cx, "bar.toml", &store.bar_path(), BAR_SETTINGS);
    for line in bar_click_lines(&store) {
        say!(cx, "{line}");
    }
    if let Some(line) = store.try_load_bar().ok().as_ref().and_then(off_line) {
        say!(cx, "{line}");
    }
}

/// What doctor says about `[bar] provider`: the bar in use, why when it is
/// not the one written, and that `bar.toml` is not read under any bar but
/// the built-in one.
fn provider_lines(provider: &ProviderCheck) -> Vec<String> {
    let name = provider.choice().name();
    let mut lines = vec![match provider.warning() {
        Some(warning) => format!("  {WARN} Bar: {warning}"),
        None => format!("  {OK} Bar: {name}"),
    }];
    if !provider.choice().is_builtin() {
        lines.push(format!(
            "  {OK} bar.toml is not read: the bar is \"{name}\", not mosaico's own"
        ));
    }
    lines
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

/// The suggestion for a `bar` that is turned off: the provider says the
/// same where mosaico looks first, and spares reading `bar.toml`.
fn off_line(bar: &BarConfig) -> Option<String> {
    (!bar.enabled).then(|| {
        format!(
            "  {WARN} bar.toml turns the bar off: [bar] provider = \"none\" in config.toml \
             says the same, and bar.toml is then not read"
        )
    })
}

#[cfg(test)]
#[path = "tests/doctor_bar_tests.rs"]
mod tests;

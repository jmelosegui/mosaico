//! `mosaico bar setup <bar>`: what a bar another program draws needs in
//! its own config to show mosaico, and picking it in `[bar] provider`
//! once it runs. mosaico never edits the other program's config.

use mosaico_core::config::schema::SettingValue;
use mosaico_core::config::{BarChoice, BarProgram};

use super::context::{Context, Exit, complain, say};

/// YASB widgets that show mosaico, for its `config.yaml`.
const YASB_WIDGETS: &str = include_str!("bar_setup_yasb.yaml");

/// Prints what `bar` needs to show mosaico, then picks it in
/// `config.toml` when its program runs. While it does not run, the
/// built-in bar stays, so the screen is not left without one.
pub fn execute(cx: &mut Context, bar: &str) -> Exit {
    let Some((choice, program, widgets)) = known(bar) else {
        let names: Vec<&str> = (BarChoice::ALL.iter())
            .filter(|c| widgets_for(**c).is_some())
            .map(|c| c.name())
            .collect();
        complain!(
            cx,
            "mosaico has no setup for a bar named \"{bar}\"; it has one for {}",
            names.join(", ")
        );
        return Exit::Failure;
    };
    let title = program.title;
    say!(cx, "{}\n", widgets.trim_end());
    if cx.config.bar_provider().choice() == choice {
        say!(cx, "[bar] provider is already \"{}\".", program.bar);
        return Exit::Success;
    }
    if cx.processes.running(program.program).is_empty() {
        say!(
            cx,
            "{title} is not running, so mosaico's bar stays for now. Start {title}, then run \
             `mosaico bar setup {}` again to pick it.",
            program.bar
        );
        return Exit::Success;
    }
    let picked = cx.config.open_settings().and_then(|mut file| {
        file.set("bar.provider", &SettingValue::Text(program.bar.to_string()))?;
        file.save()
    });
    match picked {
        Ok(()) => {
            say!(
                cx,
                "[bar] provider is now \"{}\": mosaico's bar gives way to {title}.",
                program.bar
            );
            Exit::Success
        }
        Err(e) => {
            complain!(cx, "Could not pick {title} in config.toml: {e}");
            Exit::Failure
        }
    }
}

/// The bar named `bar` that mosaico can set up, its program and the
/// widgets that show mosaico in it.
fn known(bar: &str) -> Option<(BarChoice, &'static BarProgram, &'static str)> {
    let choice = BarChoice::parse(bar)?;
    Some((choice, choice.program()?, widgets_for(choice)?))
}

/// The widgets that show mosaico in the bar `choice` names.
fn widgets_for(choice: BarChoice) -> Option<&'static str> {
    match choice {
        BarChoice::Yasb => Some(YASB_WIDGETS),
        BarChoice::Mosaico | BarChoice::External | BarChoice::Off => None,
    }
}

#[cfg(test)]
#[path = "tests/bar_setup_tests.rs"]
mod tests;

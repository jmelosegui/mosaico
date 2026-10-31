//! Closing the window, and bringing it back.

use crate::support::*;

/// The commands the last frame sent to the window.
fn commands(h: &Harness<'static, SettingsApp>) -> Vec<ViewportCommand> {
    h.output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .map(|output| output.commands.clone())
        .unwrap_or_default()
}

/// A window with one unsaved edit whose close button was just clicked.
fn closing_with_an_edit() -> Harness<'static, SettingsApp> {
    let mut h = stopped();
    h.get_by_role_and_label(Role::Switch, "Write a log file")
        .click();
    h.run();
    h.get_by_role_and_label(Role::Button, "Close").click();
    h.step();
    h
}

#[test]
fn closing_with_no_edits_closes_at_once() {
    let mut h = stopped();

    h.get_by_role_and_label(Role::Button, "Close").click();
    h.step();

    assert!(commands(&h).contains(&ViewportCommand::Close));
    assert!(h.query_by_label("Discard unsaved changes?").is_none());
}

#[test]
fn closing_with_edits_asks_first() {
    let mut h = closing_with_an_edit();

    assert!(!commands(&h).contains(&ViewportCommand::Close));
    h.run();
    h.get_by_label("Discard unsaved changes?");
}

#[test]
fn keep_editing_keeps_the_window_and_the_edits() {
    let mut h = closing_with_an_edit();
    h.run();

    h.get_by_role_and_label(Role::Button, "Keep editing")
        .click();
    h.run();

    assert!(h.query_by_label("Discard unsaved changes?").is_none());
    assert_eq!(logging(&h), &SettingValue::Bool(true));
    h.get_by_label("1 unsaved change");
}

#[test]
fn discarding_closes_the_window() {
    let mut h = closing_with_an_edit();
    h.run();

    h.get_by_role_and_label(Role::Button, "Discard and close")
        .click();
    h.step();

    assert!(commands(&h).contains(&ViewportCommand::Close));
}

#[test]
fn a_close_from_the_system_with_edits_is_held_back_to_ask() {
    let mut h = stopped();
    h.get_by_role_and_label(Role::Switch, "Write a log file")
        .click();
    h.run();

    let root = h.input_mut().viewports.entry(ViewportId::ROOT).or_default();
    root.events.push(ViewportEvent::Close);
    h.step();

    assert!(commands(&h).contains(&ViewportCommand::CancelClose));
    h.run();
    h.get_by_label("Discard unsaved changes?");
}
#[test]
fn a_summons_restores_and_focuses_the_window() {
    let mut h = stopped();
    let forward = Forward::default();
    forward.attach(&h.ctx);

    forward.bring();
    h.step();

    let sent = commands(&h);
    assert!(
        sent.contains(&ViewportCommand::Minimized(false)),
        "{sent:?}"
    );
    assert!(sent.contains(&ViewportCommand::Focus), "{sent:?}");
}

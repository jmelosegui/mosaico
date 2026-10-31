//! The General page and the edits every page shares.

use crate::support::*;

#[test]
fn toggling_a_setting_counts_one_unsaved_change() {
    let mut h = stopped();

    h.get_by_role_and_label(Role::Switch, "Write a log file")
        .click();
    h.run();

    assert_eq!(logging(&h), &SettingValue::Bool(true));
    h.get_by_label("1 unsaved change");
}

#[test]
fn revert_drops_every_edit() {
    let mut h = stopped();
    h.get_by_role_and_label(Role::Switch, "Write a log file")
        .click();
    h.run();
    h.get_by_role_and_label(Role::Switch, "Start at sign-in")
        .click();
    h.run();
    h.get_by_label("2 unsaved changes");

    h.get_by_role_and_label(Role::Button, "Revert").click();
    h.run();

    assert_eq!(h.state().edit(), &saved());
    h.get_by_label("No unsaved changes");
}

#[test]
fn picking_a_flavor_previews_it() {
    let mut h = stopped();

    h.get_by_role_and_label(Role::RadioButton, "Latte").click();
    h.run();

    let latte = Theme::Catppuccin(CatppuccinFlavor::Latte);
    assert_eq!(h.state().edit().theme(), latte);
    assert!(!h.ctx.global_style().visuals.dark_mode);
}

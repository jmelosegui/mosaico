use super::*;
use crate::commands::test_support::Harness;

/// What `config.toml` sets the bar to, if anything.
fn provider(h: &Harness) -> Option<SettingValue> {
    h.config.open_settings().ok()?.get("bar.provider")
}

#[test]
fn a_running_bar_is_picked_after_its_widgets_are_printed() {
    // Arrange
    let mut h = Harness::new("bar-setup-running");
    h.write_config("[layout]\ngap = 4\n");
    h.world.runs(5320, "yasb");

    // Act
    let exit = h.run(|cx| execute(cx, "YASB"));

    // Assert
    assert_eq!(exit, Exit::Success);
    let out = h.out();
    assert!(out.starts_with("# mosaico for YASB"), "{out}");
    assert!(out.contains("run_cmd: \"mosaico query --field focus.workspace\""));
    assert!(out.ends_with("[bar] provider is now \"yasb\": mosaico's bar gives way to YASB.\n"));
    assert_eq!(provider(&h), Some(SettingValue::Text("yasb".into())));
}

#[test]
fn a_bar_that_is_not_running_is_not_picked_so_the_screen_keeps_one() {
    // Arrange
    let mut h = Harness::new("bar-setup-stopped");
    h.world.runs(77, "yasbc");

    // Act
    let exit = h.run(|cx| execute(cx, "yasb"));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(h.out().ends_with(
        "YASB is not running, so mosaico's bar stays for now. Start YASB, then run \
         `mosaico bar setup yasb` again to pick it.\n"
    ));
    assert_eq!(provider(&h), None);
}

#[test]
fn a_bar_already_picked_is_left_as_it_is() {
    // Arrange
    let mut h = Harness::new("bar-setup-picked");
    h.write_config("[bar]\nprovider = \"yasb\"\n");

    // Act
    let exit = h.run(|cx| execute(cx, "yasb"));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(h.out().ends_with("[bar] provider is already \"yasb\".\n"));
}

#[test]
fn a_bar_without_a_setup_is_refused_with_those_that_have_one() {
    // Arrange
    let mut h = Harness::new("bar-setup-unknown");

    // Act
    let exit = h.run(|cx| execute(cx, "external"));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(
        h.err(),
        "mosaico has no setup for a bar named \"external\"; it has one for yasb\n"
    );
    assert_eq!(h.out(), "");
}

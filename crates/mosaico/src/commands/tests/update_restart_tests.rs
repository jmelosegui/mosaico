use super::tests::released;
use super::*;

#[test]
fn the_restarted_daemon_is_announced_with_the_installed_version() {
    // Arrange -- updating from an older version, which restarts the
    // daemon from the process that is still the old version.
    let mut h = released("update-banner", "v99.0.0");
    h.world.daemon_up(10);

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(*h.world.logos.borrow(), ["99.0.0"]);
    let out = h.out();
    assert!(!out.contains("Update available"), "{out}");
}

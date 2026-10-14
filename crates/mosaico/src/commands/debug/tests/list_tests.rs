use mosaico_core::WindowId;

use super::*;
use crate::commands::fake_desktop::{app_window, monitor};
use crate::commands::test_support::Harness;

fn wid(raw: u64) -> WindowId {
    WindowId::from_raw(raw)
}

/// The `Managed: ..  Monitor: ..  State: ..` line of each window, in
/// order.
fn status_lines(out: &str) -> Vec<&str> {
    out.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("Managed:"))
        .collect()
}

/// A rule that tiles windows of class `App`.
fn tile_app() -> WindowRule {
    WindowRule {
        match_class: Some("App".into()),
        match_title: None,
        match_path: None,
        manage: true,
    }
}

#[test]
fn prints_each_window_and_monitor() {
    // Arrange
    let mut h = Harness::new("list-format");
    h.desktop.monitors = vec![monitor(1, 0), monitor(2, 1920)];
    h.desktop.windows = vec![(wid(0x4014E), app_window(2))];

    // Act
    let exit = h.run(|cx| list(cx, &[]));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(
        h.out(),
        "  0x4014E  app\n\
         \x20      Class: App\n\
         \x20      Path: C:\\apps\\app.exe\n\
         \x20      Managed: yes  Monitor: 2  State: normal\n\
         \x20      Rect: 640x480 at (100, 100)  Border: L:7 T:0 R:7 B:7\n\
         \n\
         1 windows found\n\
         \n\
         \x20 Monitor 1 (primary): 1920x1040 at (0, 0)\n\
         \x20      Key: [mon1]\n\
         \x20 Monitor 2: 1920x1040 at (1920, 0)\n\
         \x20      Key: [mon2]\n"
    );
}

#[test]
fn numbers_monitors_from_the_left_whatever_order_the_platform_finds_them_in() {
    // Arrange -- the platform finds the right monitor first.
    let mut h = Harness::new("list-order");
    h.desktop.monitors = vec![monitor(2, 1920), monitor(1, 0)];
    h.desktop.windows = vec![(wid(1), app_window(1)), (wid(2), app_window(2))];

    // Act
    h.run(|cx| list(cx, &[]));

    // Assert
    let out = h.out();
    assert_eq!(
        status_lines(&out),
        [
            "Managed: yes  Monitor: 1  State: normal",
            "Managed: yes  Monitor: 2  State: normal",
        ]
    );
    let left = out.find("Monitor 1 (primary): 1920x1040 at (0, 0)");
    let right = out.find("Monitor 2: 1920x1040 at (1920, 0)");
    assert!(left.is_some() && left < right, "{out}");
}

#[test]
fn a_window_on_a_monitor_that_is_not_listed_has_no_monitor_number() {
    // Arrange
    let mut h = Harness::new("list-unknown-monitor");
    h.desktop.monitors = vec![monitor(1, 0)];
    h.desktop.windows = vec![(wid(1), app_window(9))];

    // Act
    h.run(|cx| list(cx, &[]));

    // Assert
    assert_eq!(
        status_lines(&h.out()),
        ["Managed: yes  Monitor: ?  State: normal"]
    );
}

#[test]
fn a_window_without_a_title_bar_is_managed_only_when_a_rule_asks_for_it() {
    // Arrange
    let mut frameless = app_window(1);
    frameless.facts.has_caption = false;
    let mut without_rule = Harness::new("list-frameless");
    let mut with_rule = Harness::new("list-frameless-rule");
    for h in [&mut without_rule, &mut with_rule] {
        h.desktop.monitors = vec![monitor(1, 0)];
        h.desktop.windows = vec![(wid(1), frameless.clone())];
    }

    // Act
    without_rule.run(|cx| list(cx, &[]));
    with_rule.run(|cx| list(cx, &[tile_app()]));

    // Assert
    assert_eq!(
        status_lines(&without_rule.out()),
        ["Managed: no  Monitor: 1  State: normal"]
    );
    assert_eq!(
        status_lines(&with_rule.out()),
        ["Managed: yes  Monitor: 1  State: normal"]
    );
}

#[test]
fn a_frame_that_cannot_be_read_shows_as_a_question_mark() {
    // Arrange
    let mut window = app_window(1);
    window.frame = None;
    let mut h = Harness::new("list-no-frame");
    h.desktop.monitors = vec![monitor(1, 0)];
    h.desktop.windows = vec![(wid(1), window)];

    // Act
    h.run(|cx| list(cx, &[]));

    // Assert
    assert!(
        h.out().contains("Rect: 640x480 at (100, 100)  Border: ?\n"),
        "{}",
        h.out()
    );
}

#[test]
fn fails_with_the_reason_when_the_windows_cannot_be_listed() {
    // Arrange
    let mut h = Harness::new("list-unreadable");
    h.desktop.monitors = vec![monitor(1, 0)];
    h.desktop.windows_unreadable = true;

    // Act
    let exit = h.run(|cx| list(cx, &[]));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.err(), "Error: could not list windows: access denied\n");
    assert_eq!(h.out(), "");
}

#[test]
fn lists_the_windows_without_monitor_numbers_when_the_monitors_cannot_be_listed() {
    // Arrange
    let mut h = Harness::new("list-monitors-unreadable");
    h.desktop.monitors = vec![monitor(1, 0)];
    h.desktop.windows = vec![(wid(1), app_window(1)), (wid(2), app_window(1))];
    h.desktop.monitors_unreadable = true;

    // Act
    let exit = h.run(|cx| list(cx, &[]));

    // Assert
    assert_eq!(exit, Exit::Success);
    let out = h.out();
    assert_eq!(
        status_lines(&out),
        [
            "Managed: yes  Monitor: ?  State: normal",
            "Managed: yes  Monitor: ?  State: normal",
        ]
    );
    assert!(out.ends_with("2 windows found\n\n"), "{out}");
    assert_eq!(h.err(), "");
}

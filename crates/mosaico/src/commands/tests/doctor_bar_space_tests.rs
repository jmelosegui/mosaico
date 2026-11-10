use mosaico_core::config::ReserveEdges;
use mosaico_core::{Rect, StableMonitorId};

use super::*;
use crate::commands::context::Exit;
use crate::commands::fake_desktop::monitor;
use crate::commands::test_support::Harness;

/// Monitor `index` from the left, 1920x1080 at 100%, whose taskbar keeps
/// the bottom 40 pixels.
fn place(index: usize) -> MonitorPlace {
    let x = index as i32 * 1920;
    MonitorPlace {
        id: StableMonitorId::new(format!("m{index}")),
        index,
        bounds: Rect::new(x, 0, 1920, 1080),
        work_area: Rect::new(x, 0, 1920, 1040),
        scale: 1.0,
    }
}

/// A `[bar.reserve]` keeping `top` on every monitor.
fn top(top: i32) -> BarReserve {
    BarReserve {
        top,
        ..BarReserve::default()
    }
}

#[test]
fn under_another_programs_bar_each_monitor_says_what_the_reserve_keeps() {
    // Act
    let lines = space_lines(BarChoice::External, &top(32), &[place(0), place(1)]);

    // Assert
    assert_eq!(
        lines,
        [
            format!("  {OK} Space tiles keep clear of:"),
            "       On monitor 0 (m0): the system keeps bottom 40; [bar.reserve] keeps top 32"
                .into(),
            "       On monitor 1 (m1): the system keeps bottom 40; [bar.reserve] keeps top 32"
                .into(),
        ]
    );
}

#[test]
fn an_edge_past_a_quarter_and_a_table_for_no_monitor_are_flagged() {
    // Arrange
    let mut reserve = top(0);
    let tall = ReserveEdges {
        left: 600,
        ..ReserveEdges::default()
    };
    reserve.monitor.insert("0".into(), tall);
    reserve.monitor.insert("gone".into(), tall);

    // Act
    let lines = space_lines(BarChoice::External, &reserve, &[place(0)]);

    // Assert
    assert_eq!(
        lines[1..],
        [
            "       On monitor 0 (m0): the system keeps bottom 40; [bar.reserve] keeps nothing"
                .to_string(),
            format!(
                "  {WARN} [bar.reserve] keeps nothing at the left of monitor 0: it would take \
                 more than a quarter of the monitor"
            ),
            format!(
                "  {WARN} [bar.reserve.monitor.gone] keeps nothing: no connected monitor has \
                 that id or position"
            ),
        ]
    );
}

#[test]
fn under_mosaicos_bar_the_reserve_is_said_to_be_unused() {
    // Act
    let lines = space_lines(BarChoice::Mosaico, &top(32), &[place(0)]);

    // Assert
    assert_eq!(
        lines[1..],
        [
            "       On monitor 0 (m0): the system keeps bottom 40; mosaico's bar keeps the space it \
             draws in"
                .to_string(),
            format!("  {OK} [bar.reserve] is not used: the bar is \"mosaico\", not \"external\""),
        ]
    );
}

#[test]
fn with_no_bar_only_the_system_keeps_space() {
    // Act
    let lines = space_lines(BarChoice::Off, &BarReserve::default(), &[place(0)]);

    // Assert
    assert_eq!(
        lines[1..],
        ["       On monitor 0 (m0): the system keeps bottom 40".to_string()]
    );
}

#[test]
fn the_check_reads_the_reserve_from_the_config_file() {
    // Arrange
    let mut h = Harness::new("doctor-bar-space");
    h.desktop.monitors = vec![monitor(1, 0)];
    std::fs::create_dir_all(h.config.dir()).unwrap();
    let toml = "[bar]\nprovider = \"external\"\n\n[bar.reserve]\nright = 24\n";
    std::fs::write(h.config.config_path(), toml).unwrap();

    // Act
    h.run(|cx| {
        check_space(cx, BarChoice::External);
        Exit::Success
    });
    let _ = std::fs::remove_dir_all(h.config.dir());

    // Assert
    assert!(
        h.out().contains("[bar.reserve] keeps right 24"),
        "{}",
        h.out()
    );
}

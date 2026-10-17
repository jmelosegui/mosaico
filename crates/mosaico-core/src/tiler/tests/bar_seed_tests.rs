//! Windows that fill their monitor already when the tiler starts, or when
//! the bars are made again, without the tiler seeing them move: once they
//! stop, the bars must be drawn again too.

use std::time::Instant;

use crate::WindowEvent;
use crate::id::wid;
use crate::tiler::tests::fake_desktop::{FakeDesktop, FakeWindow};
use crate::tiler::tests::{Harness, displays};
use crate::tiler::{BarReservation, Input, Settings, Tiler};

/// A window on monitor 0 that the tiler leaves alone.
fn unmanaged() -> FakeWindow {
    let mut window = FakeWindow::app(0);
    window.facts = None;
    window
}

/// A tiler started on one monitor with window 1, which it takes in, and
/// window 9, which it leaves alone, with `maximized` maximized already, as
/// at login or when the daemon starts again.
fn started_with(maximized: u64) -> (Tiler, FakeDesktop) {
    let mut desktop = FakeDesktop::default();
    desktop.windows.insert(wid(1), FakeWindow::app(0));
    desktop.windows.insert(wid(9), unmanaged());
    desktop
        .windows
        .get_mut(&wid(maximized))
        .expect("a window")
        .maximized = true;
    let windows = [wid(1), wid(9)];
    let (tiler, _) = Tiler::new(
        Settings::default(),
        &displays(1),
        &windows,
        &desktop,
        Instant::now(),
    );
    (tiler, desktop)
}

/// Restores `window`, reports that it moved, and says whether the bars
/// must be drawn again.
fn restored(tiler: &mut Tiler, desktop: &mut FakeDesktop, window: u64) -> bool {
    desktop
        .windows
        .get_mut(&wid(window))
        .expect("a window")
        .maximized = false;
    let moved = WindowEvent::LocationChanged {
        window: wid(window),
    };
    tiler.step(Input::Event(moved), &*desktop, Instant::now());
    tiler.take_bars_changed()
}

#[test]
fn an_unmanaged_window_maximized_as_the_tiler_starts_counts_once_restored() {
    // Arrange
    let (mut tiler, mut desktop) = started_with(9);

    // Act
    let changed = restored(&mut tiler, &mut desktop, 9);

    // Assert
    assert!(changed);
}

#[test]
fn a_managed_window_maximized_as_the_tiler_starts_counts_once_restored() {
    // Arrange
    let (mut tiler, mut desktop) = started_with(1);

    // Act
    let changed = restored(&mut tiler, &mut desktop, 1);

    // Assert
    assert!(changed);
}

#[test]
fn a_window_maximized_as_the_bars_are_made_again_counts_once_restored() {
    // Arrange -- window 9 is maximized without being seen to move, and
    // then the bar takes its space again, as when bar.toml is reloaded.
    let mut h = Harness::new(1, &[(1, 0)]);
    let mut player = unmanaged();
    player.maximized = true;
    h.desktop.windows.insert(wid(9), player);
    h.step(Input::Bar(BarReservation {
        height: 30,
        monitors: Vec::new(),
    }));

    // Act
    h.window(9).maximized = false;
    h.event(WindowEvent::LocationChanged { window: wid(9) });

    // Assert
    assert!(h.tiler.take_bars_changed());
}

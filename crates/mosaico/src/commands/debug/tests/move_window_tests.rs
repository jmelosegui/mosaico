use clap::Parser;

use mosaico_core::{Rect, WindowId};

use super::*;
use crate::commands::test_support::Harness;

fn args(window: &str) -> MoveArgs {
    MoveArgs {
        window: window.into(),
        x: 10,
        y: 20,
        w: 300,
        h: 200,
    }
}

#[test]
fn places_the_window_at_the_requested_bounds() {
    // Arrange
    let mut h = Harness::new("move-places");
    let window = WindowId::parse("0x4014E").expect("a window id");

    // Act
    let exit = h.run(|cx| execute(cx, &args("0x4014E")));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(
        *h.world.placed.borrow(),
        [(window, Rect::new(10, 20, 300, 200))]
    );
    assert_eq!(
        h.out(),
        "Moving window 0x4014E to (10, 20) 300x200\nDone.\n"
    );
}

#[test]
fn refuses_text_that_is_not_a_window_id() {
    // Arrange
    let mut h = Harness::new("move-not-an-id");

    // Act
    let exit = h.run(|cx| execute(cx, &args("notepad")));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert!(h.world.placed.borrow().is_empty());
    assert_eq!(h.out(), "");
    assert!(
        h.err().starts_with("Not a window id: notepad"),
        "{}",
        h.err()
    );
}

#[test]
fn fails_when_the_window_cannot_be_moved() {
    // Arrange
    let mut h = Harness::new("move-fails");
    *h.world.place_error.borrow_mut() = Some("the window is gone".into());

    // Act
    let exit = h.run(|cx| execute(cx, &args("0x4014E")));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(
        h.err(),
        "Error: could not move the window: the window is gone\n"
    );
}

#[test]
fn still_takes_the_window_as_hwnd() {
    // Arrange
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        args: MoveArgs,
    }
    let argv = [
        "move", "--hwnd", "0x4014E", "--x", "1", "--y", "2", "--w", "3", "--h", "4",
    ];

    // Act
    let cli = Cli::try_parse_from(argv).expect("--hwnd is still accepted");

    // Assert
    assert_eq!(cli.args.window, "0x4014E");
}

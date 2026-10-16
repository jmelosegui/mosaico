use clap::Args;

use mosaico_core::{Rect, WindowId};

use crate::commands::context::{Context, Exit, complain, say};

/// Arguments for the `debug move` subcommand.
#[derive(Args)]
pub struct MoveArgs {
    /// Window id as `debug list` shows it (hex with 0x prefix, or decimal)
    #[arg(long, alias = "hwnd")]
    window: String,
    /// X position in pixels
    #[arg(long)]
    x: i32,
    /// Y position in pixels
    #[arg(long)]
    y: i32,
    /// Width in pixels
    #[arg(long, name = "width")]
    w: i32,
    /// Height in pixels
    #[arg(long, name = "height")]
    h: i32,
}

/// Moves one window so its visible bounds are the given rect.
///
/// It uses the same `WindowControl::place` as the tiler's placements,
/// but does not skip a window that is already there, and does not
/// repaint it afterwards.
pub fn execute(cx: &mut Context, args: &MoveArgs) -> Exit {
    let Some(window) = WindowId::parse(&args.window) else {
        complain!(
            cx,
            "Not a window id: {} (use one `mosaico debug list` shows)",
            args.window
        );
        return Exit::Failure;
    };
    let target = Rect::new(args.x, args.y, args.w, args.h);

    say!(
        cx,
        "Moving window {window} to ({}, {}) {}x{}",
        target.x,
        target.y,
        target.width,
        target.height
    );

    match cx.windows.place(window, target) {
        Ok(()) => {
            say!(cx, "Done.");
            Exit::Success
        }
        Err(e) => {
            complain!(cx, "Error: {e}");
            Exit::Failure
        }
    }
}

#[cfg(test)]
#[path = "tests/move_window_tests.rs"]
mod tests;

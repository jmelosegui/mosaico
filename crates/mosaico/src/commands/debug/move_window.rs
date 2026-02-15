use clap::Args;

use mosaico_core::{Rect, window::Window};

/// Arguments for the `debug move` subcommand.
#[derive(Args)]
pub struct MoveArgs {
    /// Window handle (decimal or hex with 0x prefix)
    #[arg(long)]
    hwnd: String,
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

pub fn execute(args: &MoveArgs) {
    mosaico_windows::dpi::enable_dpi_awareness();

    let hwnd_val = parse_hwnd(&args.hwnd);
    let window = mosaico_windows::Window::from_raw(hwnd_val);
    let target = Rect::new(args.x, args.y, args.w, args.h);

    println!(
        "Moving window 0x{hwnd_val:X} to ({}, {}) {}x{}",
        target.x, target.y, target.width, target.height
    );

    match window.set_rect(&target) {
        Ok(()) => println!("Done."),
        Err(e) => eprintln!("Failed to move window: {e}"),
    }
}

/// Parses a window handle from a string (supports decimal and 0x hex).
fn parse_hwnd(s: &str) -> usize {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        usize::from_str_radix(hex, 16).expect("invalid hex hwnd")
    } else {
        s.parse().expect("invalid decimal hwnd")
    }
}

use std::io::{Write, stdout};
use std::thread;
use std::time::Duration;

/// The mosaico logo in 6-row block-letter style matching the aspire CLI font.
const LOGO: [&str; 6] = [
    "██▄ ▄██  █████  ███████  █████  ██ ██████  █████ ",
    "██ █ ██ ██▀▀▀██ ██▀▀▀▀▀ ██▀▀▀██ ██ ██▀▀▀▀ ██▀▀▀██",
    "██ ▀ ██ ██   ██ ███████ ███████ ██ ██     ██   ██",
    "██   ██ ██   ██ ▀▀▀▀▀██ ██   ██ ██ ██     ██   ██",
    "██   ██  █████  ███████ ██   ██ ██ ██████  █████ ",
    "▀▀   ▀▀  ▀▀▀▀▀  ▀▀▀▀▀▀▀ ▀▀   ▀▀ ▀▀ ▀▀▀▀▀▀  ▀▀▀▀▀ ",
];

/// ANSI color constants.
const BORDER: &str = "\x1b[38;5;8m";
const BODY: &str = "\x1b[38;2;137;180;250m";    // Catppuccin Mocha blue #89b4fa
const SHADOW: &str = "\x1b[38;2;180;190;254m"; // Catppuccin Mocha lavender #b4befe
const SHINE: &str = "\x1b[38;2;245;224;220m";  // Catppuccin Mocha rosewater #f5e0dc
const HEADER: &str = "\x1b[38;5;15m";
const RESET: &str = "\x1b[0m";

/// Logo width in characters (longest line).
const LOGO_WIDTH: usize = 49;

/// Inner width of the box (2 padding each side + logo).
const INNER: usize = LOGO_WIDTH + 4;

/// Total inner rows: blank, header, 6 logo, version, blank = 10.
const ROWS: usize = 10;

/// Draws one frame of the box, overwriting the previous one.
fn draw_frame(
    out: &mut impl Write,
    frame_idx: usize,
    header: &str,
    logo_cols: usize,
    ver: &str,
    show_ver: bool,
    shine_col: Option<usize>,
) {
    if frame_idx > 0 {
        write!(out, "\x1b[{}A", ROWS + 2).unwrap();
    }

    let logo_chars: Vec<Vec<char>> = LOGO.iter().map(|l| l.chars().collect()).collect();

    // Top border.
    writeln!(out, "{BORDER}╭{}╮{RESET}", "─".repeat(INNER)).unwrap();

    // Row 0: blank.
    writeln!(out, "{BORDER}│{RESET}{}{BORDER}│{RESET}", " ".repeat(INNER)).unwrap();

    // Row 1: header text.
    let h_pad = INNER - 2 - header.len();
    writeln!(
        out,
        "{BORDER}│{RESET}  {HEADER}{header}{RESET}{}{BORDER}│{RESET}",
        " ".repeat(h_pad)
    )
    .unwrap();

    // Rows 2-7: logo lines.
    for row_chars in &logo_chars {
        let visible: String = row_chars.iter().take(logo_cols).collect();
        let vis_len = visible.chars().count();
        let pad = INNER - 2 - vis_len;
        let colored = colorize_row(&visible, shine_col);
        writeln!(
            out,
            "{BORDER}│{RESET}  {colored}{}{BORDER}│{RESET}",
            " ".repeat(pad)
        )
        .unwrap();
    }

    // Row 8: version (right-aligned to logo edge).
    if show_ver {
        let logo_right_pad = INNER - 2 - LOGO_WIDTH;
        let ver_pad = 2 + LOGO_WIDTH - ver.len();
        writeln!(
            out,
            "{BORDER}│{RESET}{}{HEADER}{ver}{RESET}{}{BORDER}│{RESET}",
            " ".repeat(ver_pad),
            " ".repeat(logo_right_pad)
        )
        .unwrap();
    } else {
        writeln!(out, "{BORDER}│{RESET}{}{BORDER}│{RESET}", " ".repeat(INNER)).unwrap();
    }

    // Row 9: blank.
    writeln!(out, "{BORDER}│{RESET}{}{BORDER}│{RESET}", " ".repeat(INNER)).unwrap();

    // Bottom border.
    writeln!(out, "{BORDER}╰{}╯{RESET}", "─".repeat(INNER)).unwrap();

    out.flush().unwrap();
}

/// Prints the animated mosaico banner matching the aspire CLI style.
///
/// Animation stages:
/// 1. Empty panel (80ms)
/// 2. Header typewriter — 3 chars at a time (40ms per step)
/// 3. Logo letter-by-letter (70ms per letter)
/// 4. Version slide-in (50ms per step)
/// 5. Shine sweep across logo (35ms per column)
pub fn print_logo() {
    let mut out = stdout();
    let ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let header_full = "Welcome to";
    let total_logo_cols = LOGO.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let mut frame: usize = 0;

    // Hide cursor.
    write!(out, "\x1b[?25l").unwrap();
    out.flush().unwrap();

    // Stage 1: Empty panel.
    draw_frame(&mut out, frame, "", 0, &ver, false, None);
    frame += 1;
    thread::sleep(Duration::from_millis(80));

    // Stage 2: Header typewriter — 3 chars at a time.
    let header_chars: Vec<char> = header_full.chars().collect();
    let mut i = 0;
    while i < header_chars.len() {
        i = (i + 3).min(header_chars.len());
        let partial: String = header_chars[..i].iter().collect();
        draw_frame(&mut out, frame, &partial, 0, &ver, false, None);
        frame += 1;
        thread::sleep(Duration::from_millis(40));
    }

    // Stage 3: Logo reveal — 2 columns at a time, same rate as shine sweep.
    let mut cols = 0;
    while cols < total_logo_cols {
        cols = (cols + 2).min(total_logo_cols);
        draw_frame(&mut out, frame, header_full, cols, &ver, false, None);
        frame += 1;
        thread::sleep(Duration::from_millis(35));
    }

    // Stage 4: Version slide-in.
    draw_frame(&mut out, frame, header_full, total_logo_cols, &ver, true, None);
    frame += 1;
    thread::sleep(Duration::from_millis(50));

    // Stage 5: Shine sweep — 3-column highlight moves left to right.
    let mut col = 0;
    while col <= total_logo_cols + 3 {
        draw_frame(&mut out, frame, header_full, total_logo_cols, &ver, true, Some(col));
        frame += 1;
        col += 2;
        thread::sleep(Duration::from_millis(35));
    }

    // Stage 6: Final static frame (no shine).
    draw_frame(&mut out, frame, header_full, total_logo_cols, &ver, true, None);

    // Show cursor.
    write!(out, "\x1b[?25h").unwrap();
    out.flush().unwrap();
}

/// Colorizes a logo row with optional shine highlight.
///
/// Full blocks (█▄) use BODY, half blocks (▀) use SHADOW.
/// The shine sweep overrides with SHINE for a 3-column highlight.
fn colorize_row(line: &str, shine_col: Option<usize>) -> String {
    if line.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(line.len() * 4);
    let mut current: &str = "";

    for (col, ch) in line.chars().enumerate() {
        let in_shine = shine_col.is_some_and(|sc| col >= sc && col < sc + 3);

        let color = if in_shine {
            SHINE
        } else {
            match ch {
                '█' | '▄' => BODY,
                '▀' => SHADOW,
                _ => current,
            }
        };

        if !color.is_empty() && color != current {
            result.push_str(color);
            current = color;
        }
        result.push(ch);
    }

    if !current.is_empty() {
        result.push_str(RESET);
    }
    result
}

/// Runs the banner animation as a standalone command.
pub fn execute() {
    print_logo();
    println!();
}

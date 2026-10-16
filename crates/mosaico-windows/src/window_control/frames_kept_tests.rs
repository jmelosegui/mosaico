//! The invisible borders kept with a window's frames, at the DPI of the
//! display it is on when a placement is posted.

use crate::frame::BorderOffset;

use super::super::dpi::FrameWidths;

use super::*;

/// The frame widths Windows 11 gives at each scale, as measured: the
/// side is `SM_CXSIZEFRAME` with `SM_CXPADDEDBORDER`, 4 + 4 at 96, 4 + 5
/// at 120, 5 + 6 at 144, 5 + 7 at 168 and 5 + 8 at 192.
fn widths(dpi: i32) -> Option<FrameWidths> {
    let side = match dpi {
        96 => 8,
        120 => 9,
        144 => 11,
        168 => 12,
        192 => 13,
        _ => return None,
    };
    Some(FrameWidths { side, bottom: side })
}

/// Borders of `side` on the left, right and bottom, read at `dpi`, of a
/// window Windows stretches when `scaled`.
fn kept(side: i32, dpi: Option<i32>, scaled: bool) -> Kept {
    Kept {
        border: BorderOffset {
            left: side,
            top: 0,
            right: side,
            bottom: side,
        },
        scaled,
        dpi,
    }
}

/// The left, top, right and bottom borders of `kept` at `dpi`.
fn sides_at(kept: Kept, dpi: Option<i32>) -> [i32; 4] {
    let border = kept.border_at(dpi, widths);
    [border.left, border.top, border.right, border.bottom]
}

#[test]
fn borders_are_kept_while_the_dpi_they_were_read_at_holds() {
    // Arrange / Act / Assert -- or when either DPI is not known.
    assert_eq!(sides_at(kept(7, Some(96), false), Some(96)), [7, 0, 7, 7]);
    assert_eq!(sides_at(kept(7, None, false), Some(192)), [7, 0, 7, 7]);
    assert_eq!(sides_at(kept(7, Some(96), false), None), [7, 0, 7, 7]);
}

#[test]
fn the_borders_of_a_window_drawn_at_its_dpi_follow_the_frame_metrics() {
    // Arrange -- 7 at 96 and 8 at 120, as Windows Terminal and Chrome
    // measure, each the frame width less one.
    let at_96 = kept(7, Some(96), false);
    let at_120 = kept(8, Some(120), false);

    // Act / Assert -- not by the ratio, which gives 9, 11 and 14, and 6.
    assert_eq!(sides_at(at_96, Some(120)), [8, 0, 8, 8]);
    assert_eq!(sides_at(at_96, Some(144)), [10, 0, 10, 10]);
    assert_eq!(sides_at(at_96, Some(192)), [12, 0, 12, 12]);
    assert_eq!(sides_at(at_120, Some(96)), [7, 0, 7, 7]);
}

#[test]
fn the_borders_of_a_window_windows_stretches_follow_the_ratio_of_the_dpis() {
    // Arrange / Act / Assert -- rounded as Windows rounds, halves away
    // from zero.
    assert_eq!(
        sides_at(kept(7, Some(96), true), Some(192)),
        [14, 0, 14, 14]
    );
    assert_eq!(
        sides_at(kept(7, Some(96), true), Some(144)),
        [11, 0, 11, 11]
    );
    assert_eq!(sides_at(kept(7, Some(192), true), Some(96)), [4, 0, 4, 4]);
}

#[test]
fn borders_are_kept_where_the_frame_metrics_cannot_be_read() {
    // Arrange / Act / Assert -- 100 DPI has none in the table.
    assert_eq!(sides_at(kept(7, Some(96), false), Some(100)), [7, 0, 7, 7]);
}

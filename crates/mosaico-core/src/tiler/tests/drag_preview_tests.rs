//! The preview of where a dragged window lands, as its position reports
//! move it.

use crate::config::{Color, CornerStyle};
use crate::id::wid;
use crate::tiler::tests::{Harness, MON_W};
use crate::tiler::{Effect, PreviewStyle};
use crate::{Action, Rect, WindowEvent};

use super::super::support::{center, grab, report_at, shown, swapping};

#[test]
fn the_preview_shows_the_slot_of_the_tile_under_the_pointer() {
    // Arrange
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let right = h.rect_of(2);
    grab(&mut h, 1);

    // Act
    let effects = report_at(&mut h, 1, center(right));

    // Assert
    assert_eq!(shown(&effects), Some(right));
    assert_eq!(h.desktop.preview, Some(right));
}

#[test]
fn over_its_own_slot_the_preview_shows_that_slot() {
    // Arrange
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let left = h.rect_of(1);
    grab(&mut h, 1);

    // Act
    let effects = report_at(&mut h, 1, center(left));

    // Assert
    assert_eq!(shown(&effects), Some(left));
}

#[test]
fn on_another_monitor_the_preview_shows_the_slot_the_window_gets() {
    // Arrange -- window 4 has the right half of the second monitor.
    let windows = [(1, 0), (2, 0), (3, 1), (4, 1)];
    let mut h = Harness::with_settings(swapping(), 2, &windows);
    let right_half = h.rect_of(4);
    grab(&mut h, 1);

    // Act
    let effects = report_at(&mut h, 1, center(right_half));

    // Assert -- the top of the right half, which it takes once it is put
    // before window 4, not the tile under the pointer.
    let slot = shown(&effects).expect("a preview");
    assert_eq!((slot.x, slot.y), (right_half.x, right_half.y));
    assert!(slot.height < right_half.height);
    assert!(slot.x >= MON_W);
}

#[test]
fn over_a_monocle_workspace_of_another_monitor_the_preview_shows_its_tiled_slot() {
    // Arrange -- the second monitor shows window 3 in monocle, window 4
    // behind it.
    let windows = [(1, 0), (2, 0), (3, 1), (4, 1)];
    let mut h = Harness::with_settings(swapping(), 2, &windows);
    let behind = h.rect_of(4);
    h.event(WindowEvent::Focused { window: wid(3) });
    h.action(Action::ToggleMonocle);
    grab(&mut h, 1);

    // Act -- over where window 4 is tiled.
    let effects = report_at(&mut h, 1, center(behind));

    // Assert -- as the owner chose while testing, monocle is set aside
    // there: window 1 would go before window 4, at the top of the right
    // half.
    let slot = Rect::new(MON_W + 964, 8, 948, 528);
    assert_eq!(shown(&effects), Some(slot));
}

#[test]
fn the_preview_is_drawn_again_only_when_where_it_shows_changes() {
    // Arrange -- shown over window 2.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let right = h.rect_of(2);
    grab(&mut h, 1);
    report_at(&mut h, 1, center(right));

    // Act -- the pointer moves on over the same tile.
    let effects = report_at(&mut h, 1, (right.x + 20, right.y + 30));

    // Assert -- nothing but the dragged window's border moves.
    let previews = |e: &Effect| matches!(e, Effect::ShowPreview { .. } | Effect::HidePreview);
    assert!(!effects.iter().any(previews), "{effects:?}");
}

#[test]
fn the_preview_is_drawn_in_the_focused_border_color() {
    // Arrange -- borders 6 wide, round, in red when focused.
    let mut settings = swapping();
    settings.borders.colors.focused = "#ff0000".into();
    settings.borders.width = 6;
    settings.borders.corner_style = CornerStyle::Round;
    let mut h = Harness::with_settings(settings, 1, &[(1, 0), (2, 0)]);
    let right = h.rect_of(2);
    grab(&mut h, 1);

    // Act
    let effects = report_at(&mut h, 1, center(right));

    // Assert
    let style = PreviewStyle {
        color: Color::from_hex("#ff0000").expect("a color"),
        width: 6,
        radius: 16,
    };
    let window = wid(1);
    let expected = Effect::ShowPreview {
        window,
        rect: right,
        style,
    };
    assert!(effects.contains(&expected), "{effects:?}");
}

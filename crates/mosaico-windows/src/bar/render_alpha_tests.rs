//! The opacity of a painted bar's pixels: where it takes the pointer, and
//! where it lets it through.

use super::*;

#[test]
fn a_bar_that_takes_the_pointer_leaves_no_pixel_of_a_label_fully_transparent() {
    // Arrange
    let Some((mut canvas, regions)) = painted(true) else {
        return;
    };
    let width = canvas.w;

    // Act -- the alpha of every pixel of the labels, which take clicks.
    let pixels = canvas.pixels();
    let mut alphas = Vec::new();
    for rect in regions.iter().filter(|r| r.clickable).map(|r| r.rect) {
        for y in rect.y..rect.y + rect.height {
            let row = (y * width) as usize;
            let line = &pixels[row + rect.x as usize..row + (rect.x + rect.width) as usize];
            alphas.extend(line.iter().map(|pixel| pixel >> 24));
        }
    }

    // Assert
    assert!(!alphas.is_empty());
    assert!(alphas.iter().all(|&alpha| alpha != 0));
}

#[test]
fn the_background_between_items_stays_transparent_either_way() {
    // Arrange -- the top left corner, in the padding before the first
    // item, is background.
    let Some(((mut taking, _), (mut letting, _))) = painted(true).zip(painted(false)) else {
        return;
    };

    // Act
    let corners = (taking.pixels()[0] >> 24, letting.pixels()[0] >> 24);

    // Assert -- so the pointer goes through to the window under the bar.
    assert_eq!(corners, (0, 0));
}

#[test]
fn no_opaque_pixel_takes_the_background_color() {
    // Arrange -- GDI writes no alpha. A pixel it left the background's
    // color but made opaque would show as a black dot on the bar.
    let Some((mut canvas, _regions)) = painted(true) else {
        return;
    };
    let background = canvas.pixels()[0];

    // Act
    let stray = canvas
        .pixels()
        .iter()
        .filter(|&&pixel| pixel >> 24 == 0xFF && pixel & 0x00FF_FFFF == background & 0x00FF_FFFF)
        .count();

    // Assert
    assert_eq!(stray, 0);
}

#[test]
fn the_layout_pill_takes_the_pointer_above_and_below_it_once_a_click_is_bound() {
    // Arrange -- the default bar binds clicks on the layout widget, and its
    // pill does not reach the top of the bar, where the background shows.
    let Some(((mut taking, regions), (mut letting, _))) = painted(true).zip(painted(false)) else {
        return;
    };
    let layout = BarTarget::Widget(WidgetRef {
        side: BarSide::Left,
        index: 2,
    });
    let pill = regions
        .iter()
        .find(|r| r.target == layout)
        .expect("a layout pill");

    // Act -- the alpha of the top row, over the middle of the pill.
    let top = (pill.rect.x + pill.rect.width / 2) as usize;
    let alphas = (taking.pixels()[top] >> 24, letting.pixels()[top] >> 24);

    // Assert -- the least alpha there is, so the whole item takes clicks.
    assert!(pill.clickable);
    assert_eq!(alphas, (1, 0));
}

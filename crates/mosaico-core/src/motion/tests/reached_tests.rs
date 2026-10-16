//! Where the moving windows were found, for what follows them.

use std::time::Instant;

use crate::Rect;
use crate::frames::FrameOutcome;
use crate::id::wid;
use crate::motion::Easing;
use crate::motion::test_support::{ms, spec};

use super::Motions;

/// The left half of the screen, and the right one.
const LEFT: Rect = Rect {
    x: 0,
    y: 0,
    width: 800,
    height: 900,
};
const RIGHT: Rect = Rect { x: 800, ..LEFT };

/// A frame found handled, with the window where it was found.
fn handled(reached: Rect) -> FrameOutcome {
    FrameOutcome::Handled { reached }
}

#[test]
fn a_window_that_took_its_last_frame_exactly_is_followed_to_the_next() {
    // Arrange -- its first frame is handled, and it lands where it was
    // sent.
    let t0 = Instant::now();
    let mut motions = Motions::default();
    motions.retarget(wid(1), LEFT, RIGHT, t0, spec(180, Easing::Linear));
    let mut first = motions.frame(t0 + ms(16));
    motions.take(&mut first, &[handled(LEFT)], t0 + ms(16));
    let landed = first.in_flight[0].rect;

    // Act
    let mut second = motions.frame(t0 + ms(32));
    motions.take(&mut second, &[handled(landed)], t0 + ms(32));

    // Assert -- what follows it goes where the frame now posted takes it,
    // not where the one before did, which would trail the window.
    assert!(first.reached.is_empty());
    assert_eq!(second.reached, [(wid(1), second.in_flight[0].rect)]);
}

#[test]
fn a_window_that_took_its_last_frame_late_is_followed_to_where_it_was_found() {
    // Arrange -- its first frame is handled, and the next frame finds it
    // still to handle it.
    let t0 = Instant::now();
    let mut motions = Motions::default();
    motions.retarget(wid(1), LEFT, RIGHT, t0, spec(180, Easing::Linear));
    let mut first = motions.frame(t0 + ms(16));
    motions.take(&mut first, &[handled(LEFT)], t0 + ms(16));
    let mut second = motions.frame(t0 + ms(32));
    motions.take(&mut second, &[FrameOutcome::Pending], t0 + ms(32));

    // Act -- the frame after finds it where its first frame took it.
    let landed = first.in_flight[0].rect;
    let mut third = motions.frame(t0 + ms(48));
    motions.take(&mut third, &[handled(landed)], t0 + ms(48));

    // Assert -- what follows it goes there, not ahead of it to the frame
    // now posted, which it may take as long to get to.
    assert_eq!(third.reached, [(wid(1), landed)]);
}

#[test]
fn a_window_is_where_the_last_frame_found_it_while_it_has_its_frame_to_handle() {
    // Arrange -- its first frame is handled, its second lands somewhere
    // else than it was sent, and its third is not handled yet.
    let t0 = Instant::now();
    let mut motions = Motions::default();
    motions.retarget(wid(1), LEFT, RIGHT, t0, spec(180, Easing::Linear));
    let mut first = motions.frame(t0 + ms(16));
    motions.take(&mut first, &[handled(LEFT)], t0 + ms(16));
    let mut second = motions.frame(t0 + ms(32));
    let landed = Rect::new(LEFT.x + 7, LEFT.y, LEFT.width, LEFT.height);
    motions.take(&mut second, &[handled(landed)], t0 + ms(32));

    // Act -- then it handles the third.
    let mut third = motions.frame(t0 + ms(48));
    motions.take(&mut third, &[FrameOutcome::Pending], t0 + ms(48));
    let pending = motions.position(wid(1), t0 + ms(48));
    let mut fourth = motions.frame(t0 + ms(64));
    let caught_up = handled(third.in_flight[0].rect);
    motions.take(&mut fourth, &[caught_up], t0 + ms(64));

    // Assert -- left where it was found, its border too, while the third
    // frame is still to land; then where its motion has it again.
    assert_eq!(second.reached, [(wid(1), landed)]);
    assert!(third.reached.is_empty());
    assert_eq!(pending, Some(landed));
    let now = motions.rect_at(wid(1), t0 + ms(64));
    assert_eq!(motions.position(wid(1), t0 + ms(64)), now);
}

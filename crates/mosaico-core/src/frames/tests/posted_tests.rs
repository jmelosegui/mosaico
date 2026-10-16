//! Whether the window has been seen to move since something was posted
//! to it, and whether the last frame holds the next one back.

use std::time::Duration;

use super::*;

/// Where the window was when its frame was posted.
const FROM: Rect = Rect {
    x: 100,
    y: 100,
    width: 640,
    height: 480,
};

/// A frame posted at `when` to a window at `FROM`, seen at `at` since.
fn posted(when: Instant, at: Rect) -> LastPost {
    let mut last = LastPost::new(when, FROM);
    last.seen_at(at);
    last
}

#[test]
fn a_frame_holds_the_next_one_back_while_the_window_has_not_moved() {
    // Arrange
    let now = Instant::now();
    let last = posted(now, FROM);

    // Act / Assert -- until just short of the limit.
    assert!(last.holds_back(now));
    assert!(last.holds_back(now + PENDING_LIMIT - Duration::from_millis(1)));
}

#[test]
fn any_move_since_the_frame_was_posted_lets_the_next_one_go() {
    // Arrange -- the application kept the window wider than the frame
    // asked, as it does for a minimum size, so it never reaches it.
    let now = Instant::now();
    let last = posted(now, Rect { width: 600, ..FROM });

    // Act / Assert
    assert!(!last.holds_back(now));
    assert!(!last.unmoved());
}

#[test]
fn a_frame_holds_the_next_one_back_no_longer_than_the_limit() {
    // Arrange -- the window took it without moving, or still has it.
    let now = Instant::now();
    let last = posted(now, FROM);

    // Act / Assert -- though it may still be queued.
    assert!(!last.holds_back(now + PENDING_LIMIT));
    assert!(last.unmoved());
}

#[test]
fn a_move_seen_once_lets_the_next_one_go_even_back_where_it_was() {
    // Arrange -- the frame landed and the window was seen there.
    let now = Instant::now();
    let mut last = posted(now, Rect { x: 120, ..FROM });

    // Act -- then the window was sent back.
    last.seen_at(FROM);

    // Assert
    assert!(!last.holds_back(now));
    assert!(!last.unmoved());
}

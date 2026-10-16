//! Windows moving through their frames: together, along their curves, to
//! exact placements at the end of their motions.

use crate::frames::Frame;
use crate::id::wid;
use crate::tiler::tests::fake_platform::Call;
use crate::{Action, Direction, Rect};

use super::{REFRESH, animated, frame_by_frame, placements, posted, step, swap, two};

/// Where window `id` is in `frames`, if it is there.
fn rect_of(frames: &[Frame], id: u64) -> Option<Rect> {
    frames.iter().find(|f| f.window == wid(id)).map(|f| f.rect)
}

#[test]
fn a_swap_moves_both_windows_on_the_same_frames_and_ends_with_exact_placements() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    swap(&mut manager);
    platform.take_calls();

    // Act
    let calls = frame_by_frame(&platform, &mut manager).concat();

    // Assert -- window 2 first, as it was placed first. Once their motions
    // are over, after the last frames, each window is placed in full at
    // its exact target.
    let frames = posted(&calls);
    assert!(frames.len() > 10, "{} frames", frames.len());
    for frame in &frames {
        let windows: Vec<u64> = frame.iter().map(|f| f.window.raw()).collect();
        assert_eq!(windows, [2, 1]);
    }
    assert!(frames[0].iter().all(|f| f.first));
    assert!(frames[1..].iter().flatten().all(|f| !f.first));
    assert_eq!(placements(&calls), [(2, left), (1, right)]);
    let last_frame = calls
        .iter()
        .rposition(|c| matches!(c, Call::PlaceFrames(_)));
    let first_place = calls.iter().position(|c| matches!(c, Call::Place(..)));
    assert!(last_frame < first_place);
    assert_eq!(platform.window(1).rect, right);
}

#[test]
fn a_second_swap_mid_flight_turns_the_windows_back_without_a_jump() {
    // Arrange -- four frames into a swap, window 1 is well on its way.
    let (platform, mut manager) = two(&animated());
    let (left, right) = (platform.window(1).rect, platform.window(2).rect);
    swap(&mut manager);
    for _ in 0..4 {
        platform.advance(REFRESH);
        manager.frame();
    }
    let before = posted(&platform.take_calls());
    let turning = rect_of(&before[3], 1).expect("window 1 moved").x;

    // Act -- it is sent back.
    manager.handle_action(&Action::Move(Direction::Left));
    manager.flush_pending();
    let calls = frame_by_frame(&platform, &mut manager).concat();

    // Assert -- it goes back from where it was, less than half way on the
    // first frame, and both end where they started.
    let next = rect_of(&posted(&calls)[0], 1).expect("window 1 moves").x;
    assert!(left.x < next && next < turning, "{next} after {turning}");
    assert!(
        turning - next < (turning - left.x) / 2,
        "{next} after {turning}"
    );
    assert_eq!(platform.window(1).rect, left);
    assert_eq!(platform.window(2).rect, right);
}

#[test]
fn a_moving_windows_border_is_moved_to_where_the_window_is_after_each_frame() {
    // Arrange -- window 1's application gets to each frame a frame step
    // late, window 2's at once.
    let (platform, mut manager) = two(&animated());
    platform.edit(1, |w| w.frame_delay = 1);
    swap(&mut manager);
    platform.take_calls();

    // Act -- each border move, with where its window is once the frame
    // step that made it is over.
    let mut moves = Vec::new();
    while step(&platform, &mut manager) {
        for call in platform.take_calls() {
            if let Call::MoveBorder(window, border) = call {
                moves.push((window.raw(), border, platform.window(window.raw()).rect));
            }
        }
    }

    // Assert -- neither a frame behind window 2, which keeps up, nor ahead
    // of window 1, which does not, and both borders follow their windows.
    assert!(
        moves.iter().all(|(_, border, at)| border == at),
        "{moves:?}"
    );
    for id in [1, 2] {
        let count = moves.iter().filter(|(window, ..)| *window == id).count();
        assert!(count > 3, "{count} moves for window {id}");
    }
}

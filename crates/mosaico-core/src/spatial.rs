//! Spatial navigation helpers for BSP-tiled windows.
//!
//! These are pure functions over `(handle, Rect)` slices, making them
//! easy to unit-test without any Win32 dependency.
//!
//! The core algorithm is the same for all four directions: filter
//! candidates in the requested direction that share perpendicular
//! space, then pick the physically nearest one by edge distance.

use crate::{Direction, Rect};

/// Finds the nearest neighbor in the given direction.
///
/// 1. Filter candidates whose center is beyond `focused` in the
///    requested direction, and that share perpendicular space.
/// 2. Pick the closest by edge distance (gap between touching edges).
/// 3. Break ties by the perpendicular axis (topmost for horizontal,
///    leftmost for vertical).
pub fn find_neighbor(
    positions: &[(usize, Rect)],
    focused: &Rect,
    direction: Direction,
) -> Option<usize> {
    let horizontal = matches!(direction, Direction::Left | Direction::Right);
    let positive = matches!(direction, Direction::Right | Direction::Down);

    let candidates: Vec<_> = positions
        .iter()
        .filter(|(_, r)| {
            if horizontal {
                if positive {
                    r.center_x() > focused.center_x()
                } else {
                    r.center_x() < focused.center_x()
                }
            } else if positive {
                r.center_y() > focused.center_y()
            } else {
                r.center_y() < focused.center_y()
            }
        })
        .filter(|(_, r)| {
            if horizontal {
                focused.vertical_overlap(r) > 0
            } else {
                focused.horizontal_overlap(r) > 0
            }
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    candidates
        .iter()
        .min_by_key(|(_, r)| {
            let edge_dist = if horizontal {
                if positive {
                    r.x - (focused.x + focused.width)
                } else {
                    focused.x - (r.x + r.width)
                }
            } else if positive {
                r.y - (focused.y + focused.height)
            } else {
                focused.y - (r.y + r.height)
            };
            let tiebreaker = if horizontal {
                r.center_y()
            } else {
                r.center_x()
            };
            (edge_dist.max(0), tiebreaker)
        })
        .map(|(h, _)| *h)
}

/// Finds the best window to focus when entering a monitor.
///
/// Picks the topmost window first, breaking ties by the edge closest
/// to the direction of travel (leftmost when entering from the left,
/// rightmost when entering from the right).
pub fn find_entry(positions: &[(usize, Rect)], direction: Direction) -> Option<usize> {
    let positive = matches!(direction, Direction::Right | Direction::Down);
    positions
        .iter()
        .max_by_key(|(_, r)| {
            let x = if positive {
                -r.center_x()
            } else {
                r.center_x()
            };
            (-r.center_y(), x)
        })
        .map(|(h, _)| *h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Direction;

    // ── BSP layouts used across tests ────────────────────────────

    /// 2-window BSP on a 1920x1080 area (no gap).
    ///
    /// ```text
    /// [Win 1 ─ left half] | [Win 2 ─ right half]
    /// ```
    fn two_windows() -> Vec<(usize, Rect)> {
        vec![
            (1, Rect::new(0, 0, 960, 1080)),
            (2, Rect::new(960, 0, 960, 1080)),
        ]
    }

    /// 3-window BSP on a 1920x1080 area.
    ///
    /// ```text
    /// [Win 1 ─ left half] | [Win 2 ─ top-right]
    ///                      | [Win 3 ─ bot-right]
    /// ```
    fn three_windows() -> Vec<(usize, Rect)> {
        vec![
            (1, Rect::new(0, 0, 960, 1080)),
            (2, Rect::new(960, 0, 960, 540)),
            (3, Rect::new(960, 540, 960, 540)),
        ]
    }

    /// 4-window BSP on a 1920x1080 area.
    ///
    /// ```text
    /// [Win 1 ─ left half] | [Win 2 ─ top-right half]
    ///                      | [Win 3 ─ bot-right-L] | [Win 4 ─ bot-right-R]
    /// ```
    fn four_windows() -> Vec<(usize, Rect)> {
        vec![
            (1, Rect::new(0, 0, 960, 1080)),
            (2, Rect::new(960, 0, 960, 540)),
            (3, Rect::new(960, 540, 480, 540)),
            (4, Rect::new(1440, 540, 480, 540)),
        ]
    }

    /// 5-window BSP on a 1920x1080 area.
    ///
    /// ```text
    /// [Win 1 ─ left half] | [Win 2 ─ top-right          ]
    ///                      | [Win 3 ─ bot-R-L] | [Win 4 ─ top-bot-R-R]
    ///                      |                    | [Win 5 ─ bot-bot-R-R]
    /// ```
    fn five_windows() -> Vec<(usize, Rect)> {
        vec![
            (1, Rect::new(0, 0, 960, 1080)),     // A
            (2, Rect::new(960, 0, 960, 540)),    // B
            (3, Rect::new(960, 540, 480, 540)),  // C
            (4, Rect::new(1440, 540, 480, 270)), // D
            (5, Rect::new(1440, 810, 480, 270)), // E
        ]
    }

    // ── find_neighbor: horizontal tests ──────────────────────────

    #[test]
    fn two_win_right_from_left() {
        let pos = two_windows();
        let focused = &pos[0].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Right), Some(2));
    }

    #[test]
    fn two_win_left_from_right() {
        let pos = two_windows();
        let focused = &pos[1].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Left), Some(1));
    }

    #[test]
    fn three_win_right_from_left_picks_top() {
        let pos = three_windows();
        let focused = &pos[0].1; // left half (full height)
        // B and C both touch A's right edge (edge_dist=0).
        // Topmost tiebreaker → B.
        assert_eq!(find_neighbor(&pos, focused, Direction::Right), Some(2));
    }

    #[test]
    fn three_win_left_from_top_right() {
        let pos = three_windows();
        let focused = &pos[1].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Left), Some(1));
    }

    #[test]
    fn three_win_left_from_bot_right() {
        let pos = three_windows();
        let focused = &pos[2].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Left), Some(1));
    }

    #[test]
    fn four_win_right_from_left_picks_top() {
        let pos = four_windows();
        let focused = &pos[0].1; // left half (full height)
        // B, C, D all candidates. B and C touch A (edge_dist=0).
        // Topmost tiebreaker → B.
        assert_eq!(find_neighbor(&pos, focused, Direction::Right), Some(2));
    }

    #[test]
    fn four_win_left_from_bot_right_right_picks_immediate() {
        let pos = four_windows();
        let focused = &pos[3].1; // D: bot-right-right
        // C touches D (edge_dist=0). A is further (edge_dist=480).
        assert_eq!(find_neighbor(&pos, focused, Direction::Left), Some(3));
    }

    #[test]
    fn four_win_left_from_bot_right_left_picks_left_half() {
        let pos = four_windows();
        let focused = &pos[2].1; // C: bot-right-left
        // Only candidate to the left is A.
        assert_eq!(find_neighbor(&pos, focused, Direction::Left), Some(1));
    }

    #[test]
    fn four_win_right_from_bot_right_left_picks_sibling() {
        let pos = four_windows();
        let focused = &pos[2].1; // C: bot-right-left
        // D touches C (edge_dist=0).
        assert_eq!(find_neighbor(&pos, focused, Direction::Right), Some(4));
    }

    #[test]
    fn four_win_right_from_top_right_no_neighbor() {
        let pos = four_windows();
        let focused = &pos[1].1; // B: top-right
        // D is to the right but has no vertical overlap with B.
        assert_eq!(find_neighbor(&pos, focused, Direction::Right), None);
    }

    #[test]
    fn five_win_left_from_bot_bot_right_right_picks_immediate() {
        let pos = five_windows();
        let focused = &pos[4].1; // E: bot-bot-right-right
        // C touches E (edge_dist=0). A is further (edge_dist=480).
        assert_eq!(find_neighbor(&pos, focused, Direction::Left), Some(3));
    }

    // ── Horizontal boundary tests ────────────────────────────────

    #[test]
    fn no_neighbor_when_at_edge() {
        let pos = three_windows();
        let focused = &pos[0].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Left), None);
    }

    #[test]
    fn single_window_no_neighbor() {
        let pos = vec![(1, Rect::new(0, 0, 1920, 1080))];
        let focused = &pos[0].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Right), None);
        assert_eq!(find_neighbor(&pos, focused, Direction::Left), None);
    }

    #[test]
    fn four_win_left_from_left_half_is_none() {
        let pos = four_windows();
        let focused = &pos[0].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Left), None);
    }

    #[test]
    fn four_win_right_from_bot_right_right_is_none() {
        let pos = four_windows();
        let focused = &pos[3].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Right), None);
    }

    // ── find_entry tests ─────────────────────────────────────────

    #[test]
    fn entry_from_left_two_windows() {
        let pos = two_windows();
        assert_eq!(find_entry(&pos, Direction::Right), Some(1));
    }

    #[test]
    fn entry_from_right_two_windows() {
        let pos = two_windows();
        assert_eq!(find_entry(&pos, Direction::Left), Some(2));
    }

    #[test]
    fn entry_from_left_three_windows() {
        let pos = three_windows();
        // Topmost first → Win 2 (center_y=270).
        assert_eq!(find_entry(&pos, Direction::Right), Some(2));
    }

    #[test]
    fn entry_from_right_three_windows() {
        let pos = three_windows();
        assert_eq!(find_entry(&pos, Direction::Left), Some(2));
    }

    #[test]
    fn entry_from_right_four_windows() {
        let pos = four_windows();
        assert_eq!(find_entry(&pos, Direction::Left), Some(2));
    }

    #[test]
    fn entry_single_window() {
        let pos = vec![(42, Rect::new(0, 0, 1920, 1080))];
        assert_eq!(find_entry(&pos, Direction::Right), Some(42));
        assert_eq!(find_entry(&pos, Direction::Left), Some(42));
    }

    // ── find_neighbor: vertical tests ────────────────────────────

    #[test]
    fn four_win_up_from_bot_right_right_picks_top_right() {
        let pos = four_windows();
        let focused = &pos[3].1; // D: bot-right-right
        assert_eq!(find_neighbor(&pos, focused, Direction::Up), Some(2));
    }

    #[test]
    fn four_win_up_from_bot_right_left_picks_top_right() {
        let pos = four_windows();
        let focused = &pos[2].1; // C: bot-right-left
        assert_eq!(find_neighbor(&pos, focused, Direction::Up), Some(2));
    }

    #[test]
    fn four_win_down_from_top_right_picks_leftmost_below() {
        let pos = four_windows();
        let focused = &pos[1].1; // B: top-right
        // C and D both touch B (edge_dist=0). Leftmost → C.
        assert_eq!(find_neighbor(&pos, focused, Direction::Down), Some(3));
    }

    #[test]
    fn four_win_no_vertical_neighbor_for_left_half() {
        let pos = four_windows();
        let focused = &pos[0].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Up), None);
        assert_eq!(find_neighbor(&pos, focused, Direction::Down), None);
    }

    #[test]
    fn two_win_no_vertical_neighbor() {
        let pos = two_windows();
        let focused = &pos[0].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Up), None);
        assert_eq!(find_neighbor(&pos, focused, Direction::Down), None);
    }

    #[test]
    fn three_win_down_from_top_right() {
        let pos = three_windows();
        let focused = &pos[1].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Down), Some(3));
    }

    #[test]
    fn three_win_up_from_bot_right() {
        let pos = three_windows();
        let focused = &pos[2].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Up), Some(2));
    }

    // ── Vertical boundary tests ──────────────────────────────────

    #[test]
    fn four_win_up_from_top_right_is_none() {
        let pos = four_windows();
        let focused = &pos[1].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Up), None);
    }

    #[test]
    fn four_win_down_from_bot_right_right_is_none() {
        let pos = four_windows();
        let focused = &pos[3].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Down), None);
    }

    #[test]
    fn four_win_down_from_bot_right_left_is_none() {
        let pos = four_windows();
        let focused = &pos[2].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Down), None);
    }

    #[test]
    fn three_win_up_from_top_right_is_none() {
        let pos = three_windows();
        let focused = &pos[1].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Up), None);
    }

    #[test]
    fn three_win_down_from_bot_right_is_none() {
        let pos = three_windows();
        let focused = &pos[2].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Down), None);
    }

    #[test]
    fn three_win_no_vertical_for_left_half() {
        let pos = three_windows();
        let focused = &pos[0].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Up), None);
        assert_eq!(find_neighbor(&pos, focused, Direction::Down), None);
    }

    #[test]
    fn single_window_no_vertical_neighbor() {
        let pos = vec![(1, Rect::new(0, 0, 1920, 1080))];
        let focused = &pos[0].1;
        assert_eq!(find_neighbor(&pos, focused, Direction::Up), None);
        assert_eq!(find_neighbor(&pos, focused, Direction::Down), None);
    }
}

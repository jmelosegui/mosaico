//! The step decision on where windows go, and on how many move.

use crate::MonitorId;
use crate::tiler::Effect;
use crate::tiler::tests::Harness;

use super::{ON, animated, laid_out, slot};

#[test]
fn a_window_laid_out_on_another_monitor_snaps() {
    // Arrange -- the desktop has both windows on the first monitor.
    let effects = vec![slot(1), slot(2)];
    let mut placements = laid_out(&effects, 0);
    placements.monitors[1].1 = MonitorId::from_raw(1);

    // Act
    let marked = animated(effects, &placements, &ON, 2);

    // Assert
    assert_eq!(marked, [1]);
}

#[test]
fn a_step_that_moves_more_than_sixteen_windows_snaps_them_all() {
    // Arrange -- seventeen placements; in the second step one of them
    // leaves its window where the desktop has it, which moves nothing.
    let many: Vec<Effect> = (1..=17).map(slot).collect();
    let h = Harness::new(2, &(1..=17).map(|id| (id, 0)).collect::<Vec<_>>());
    let mut one_stays = many.clone();
    if let Effect::Place { rect, .. } = &mut one_stays[16] {
        *rect = h.rect_of(17);
    }

    // Act
    let all = animated(many.clone(), &laid_out(&many, 0), &ON, 17);
    let sixteen = animated(one_stays.clone(), &laid_out(&one_stays, 0), &ON, 17);

    // Assert -- the one in place may too: applying it moves nothing.
    assert!(all.is_empty(), "{all:?}");
    assert_eq!(sixteen, (1..=17).collect::<Vec<_>>());
}

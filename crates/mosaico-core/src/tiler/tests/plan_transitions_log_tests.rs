//! What the step decision tells the log, on the effects of made-up steps.

use crate::id::wid;
use crate::motion::{SnapLine, StepSnap, StepSnapLine};
use crate::tiler::Effect;
use crate::tiler::tests::Harness;
use crate::{MonitorId, WindowId};

use super::super::{Said, StepKind, mark};
use super::{ON, laid_out, slot};

/// What `mark` tells the log of `effects`, laid out on the first monitor,
/// on a desktop with windows 1 to `count` there.
fn said(mut effects: Vec<Effect>, step: &StepKind, count: u64) -> Vec<Said> {
    let windows: Vec<(u64, usize)> = (1..=count).map(|id| (id, 0)).collect();
    let h = Harness::new(2, &windows);
    let placements = laid_out(&effects, 0);
    mark(&mut effects, &placements, step, &h.desktop)
}

#[test]
fn the_log_hears_why_a_window_that_moves_snaps_but_not_of_one_that_stays() {
    // Arrange -- both laid out on the second monitor, and window 2 left
    // where the desktop has it.
    let h = Harness::new(2, &[(1, 0), (2, 0)]);
    let mut stays = slot(2);
    if let Effect::Place { rect, .. } = &mut stays {
        *rect = h.rect_of(2);
    }
    let mut effects = vec![slot(1), stays];
    let mut placements = laid_out(&effects, 0);
    for placement in &mut placements.monitors {
        placement.1 = MonitorId::from_raw(1);
    }

    // Act
    let lines = mark(&mut effects, &placements, &ON, &h.desktop);

    // Assert
    let window: WindowId = wid(1);
    let why = StepSnap::OtherMonitor;
    assert_eq!(lines, [Said::Placement(SnapLine { window, why })]);
}

#[test]
fn a_step_that_snaps_as_a_whole_is_told_once_and_none_of_its_placements_are() {
    // Arrange
    let many: Vec<Effect> = (1..=17).map(slot).collect();
    let two = vec![slot(1), slot(2)];
    let rebuild = StepKind {
        rebuild: true,
        ..ON
    };
    let off = StepKind {
        enabled: false,
        ..ON
    };

    // Act
    let crowded = said(many, &ON, 17);
    let rebuilt = said(two.clone(), &rebuild, 2);
    let quiet = said(two, &off, 2);

    // Assert -- transitions that are off are not told again every step.
    let step = |placements, why| [Said::Step(StepSnapLine { placements, why })];
    assert_eq!(crowded, step(17, StepSnap::TooManyWindows(17)));
    assert_eq!(rebuilt, step(2, StepSnap::Rebuild));
    assert!(quiet.is_empty(), "{quiet:?}");
}

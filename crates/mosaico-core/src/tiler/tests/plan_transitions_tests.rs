//! The step decision, on the effects of made-up steps.

use std::collections::HashSet;

use crate::config::HidingBehaviour;
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::{Effect, Timing, Transition};
use crate::{MonitorId, Rect};

use super::*;

#[path = "plan_transitions_count_tests.rs"]
mod count_tests;

/// Transitions on, in a step that does not lay out from scratch.
const ON: StepKind = StepKind {
    enabled: true,
    rebuild: false,
};

/// A placement of window `id` to a slot of its own, where it is not yet.
fn slot(id: u64) -> Effect {
    Effect::Place {
        window: wid(id),
        rect: Rect::new(id as i32 * 10, 0, 500, 500),
        timing: Timing::Now,
        transition: Transition::Snap,
    }
}

/// Every placement in `effects`, as a layout pass of monitor `mon` would
/// have made it.
fn laid_out(effects: &[Effect], mon: u64) -> Placements {
    let places = effects.iter().enumerate();
    let monitors = places
        .filter(|(_, e)| matches!(e, Effect::Place { .. }))
        .map(|(at, _)| (at, MonitorId::from_raw(mon)))
        .collect();
    Placements {
        monitors,
        taken_in: HashSet::new(),
    }
}

/// The windows whose placement `step` lets animate, on a desktop with
/// windows 1 to `count` tiled on the first of two monitors.
fn animated(
    mut effects: Vec<Effect>,
    placements: &Placements,
    step: &StepKind,
    count: u64,
) -> Vec<u64> {
    let windows: Vec<(u64, usize)> = (1..=count).map(|id| (id, 0)).collect();
    let h = Harness::new(2, &windows);
    mark(&mut effects, placements, step, &h.desktop);
    let animates = |e: &Effect| match *e {
        Effect::Place {
            window,
            transition: Transition::Animate,
            ..
        } => Some(window.raw()),
        _ => None,
    };
    effects.iter().filter_map(animates).collect()
}

#[test]
fn a_placement_that_moves_its_window_on_its_monitor_may_animate() {
    // Arrange
    let effects = vec![slot(1), slot(2)];

    // Act
    let marked = animated(effects.clone(), &laid_out(&effects, 0), &ON, 2);

    // Assert
    assert_eq!(marked, [1, 2]);
}

#[test]
fn nothing_animates_while_transitions_are_off_or_the_layout_is_rebuilt() {
    // Arrange
    let effects = vec![slot(1), slot(2)];
    let placements = laid_out(&effects, 0);
    let off = StepKind {
        enabled: false,
        ..ON
    };
    let rebuild = StepKind {
        rebuild: true,
        ..ON
    };

    // Act / Assert
    assert!(animated(effects.clone(), &placements, &off, 2).is_empty());
    assert!(animated(effects, &placements, &rebuild, 2).is_empty());
}

#[test]
fn a_window_the_step_shows_places_ahead_or_takes_in_snaps() {
    // Arrange -- window 4 is shown after its placement, as a display
    // change shows the windows that arrive.
    let how = HidingBehaviour::Cloak;
    let effects = vec![
        Effect::Show {
            window: wid(1),
            how,
        },
        slot(1),
        Effect::PrePlace {
            window: wid(2),
            rect: Rect::new(20, 0, 500, 500),
        },
        slot(2),
        slot(3),
        slot(4),
        Effect::Show {
            window: wid(4),
            how,
        },
        slot(5),
    ];
    let mut placements = laid_out(&effects, 0);
    placements.taken_in.insert(wid(3));

    // Act
    let marked = animated(effects, &placements, &ON, 5);

    // Assert
    assert_eq!(marked, [5]);
}

#[test]
fn a_window_put_behind_another_before_its_placement_snaps() {
    // Arrange -- it waits behind the window in monocle.
    let behind = Effect::PutBehind {
        window: wid(2),
        other: wid(1),
    };
    let effects = vec![slot(1), behind, slot(2)];

    // Act
    let marked = animated(effects.clone(), &laid_out(&effects, 0), &ON, 2);

    // Assert
    assert_eq!(marked, [1]);
}

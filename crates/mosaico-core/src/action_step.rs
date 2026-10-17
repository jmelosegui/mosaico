//! Stepping through the workspaces of a monitor.

/// The workspace `steps` steps away from `active`, among `count`
/// workspaces numbered from 0: forward for a positive `steps`, back for
/// a negative one, as `goto-workspace-next` and `goto-workspace-prev`
/// step, once for each step.
///
/// Past the last workspace a step goes back to the first, and before the
/// first to the last, when `wrap` is set. Without it, steps stop at
/// either end. A monitor without workspaces stays on 0.
pub fn workspace_after(active: usize, count: usize, steps: i32, wrap: bool) -> usize {
    let Some(last) = count.checked_sub(1) else {
        return 0;
    };
    let active = active.min(last);
    let distance = steps.unsigned_abs() as usize;
    if wrap {
        let distance = distance % count;
        if steps >= 0 {
            (active + distance) % count
        } else {
            (active + count - distance) % count
        }
    } else if steps >= 0 {
        active.saturating_add(distance).min(last)
    } else {
        active.saturating_sub(distance)
    }
}

#[cfg(test)]
#[path = "action_step_tests.rs"]
mod tests;

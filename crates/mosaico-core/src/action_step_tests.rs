use super::*;

#[test]
fn a_step_forward_goes_to_the_next_workspace() {
    // Act
    let next = workspace_after(2, 8, 1, false);

    // Assert
    assert_eq!(next, 3);
}

#[test]
fn a_step_back_goes_to_the_workspace_before() {
    // Act
    let prev = workspace_after(2, 8, -1, false);

    // Assert
    assert_eq!(prev, 1);
}

#[test]
fn without_wrap_a_step_past_the_last_workspace_stays_on_it() {
    // Act
    let next = workspace_after(7, 8, 1, false);

    // Assert
    assert_eq!(next, 7);
}

#[test]
fn without_wrap_a_step_before_the_first_workspace_stays_on_it() {
    // Act
    let prev = workspace_after(0, 8, -1, false);

    // Assert
    assert_eq!(prev, 0);
}

#[test]
fn with_wrap_a_step_past_the_last_workspace_goes_back_to_the_first() {
    // Act
    let next = workspace_after(7, 8, 1, true);

    // Assert
    assert_eq!(next, 0);
}

#[test]
fn with_wrap_a_step_before_the_first_workspace_goes_to_the_last() {
    // Act
    let prev = workspace_after(0, 8, -1, true);

    // Assert
    assert_eq!(prev, 7);
}

#[test]
fn several_steps_without_wrap_stop_at_either_end() {
    // Act
    let forward = workspace_after(5, 8, 6, false);
    let back = workspace_after(2, 8, -6, false);

    // Assert
    assert_eq!((forward, back), (7, 0));
}

#[test]
fn several_steps_with_wrap_go_round_as_often_as_they_need() {
    // Act -- 19 steps are two rounds of 8, and 3 more.
    let forward = workspace_after(6, 8, 19, true);
    let back = workspace_after(1, 8, -19, true);

    // Assert
    assert_eq!((forward, back), (1, 6));
}

#[test]
fn no_steps_stay_on_the_workspace() {
    // Act
    let same = workspace_after(4, 8, 0, true);

    // Assert
    assert_eq!(same, 4);
}

#[test]
fn a_monitor_without_workspaces_stays_on_the_first() {
    // Act
    let next = workspace_after(0, 0, 1, true);

    // Assert
    assert_eq!(next, 0);
}

#[test]
fn an_active_workspace_past_the_count_steps_from_the_last() {
    // Act
    let prev = workspace_after(9, 8, -1, false);

    // Assert
    assert_eq!(prev, 6);
}

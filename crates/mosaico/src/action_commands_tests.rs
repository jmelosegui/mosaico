use clap::Parser;

use super::*;

/// The `action` subcommands on their own, as `mosaico action` reads them.
#[derive(Parser)]
struct ActionLine {
    #[command(subcommand)]
    action: ActionCommands,
}

/// The action `args`, the words after `mosaico action`, name.
fn parse(args: &[&str]) -> Action {
    let line = std::iter::once("action").chain(args.iter().copied());
    ActionLine::try_parse_from(line)
        .expect("the arguments parse")
        .action
        .action()
}

#[test]
fn stepping_to_the_next_and_previous_workspace_is_an_action() {
    // Act
    let next = parse(&["go-to-workspace-next"]);
    let prev = parse(&["go-to-workspace-prev"]);

    // Assert
    assert_eq!(next, Action::GoToWorkspaceNext);
    assert_eq!(prev, Action::GoToWorkspacePrev);
}

#[test]
fn the_steps_also_go_by_the_names_key_bindings_use() {
    // Act
    let next = parse(&["goto-workspace-next"]);
    let prev = parse(&["goto-workspace-prev"]);

    // Assert
    assert_eq!(next, Action::GoToWorkspaceNext);
    assert_eq!(prev, Action::GoToWorkspacePrev);
}

#[test]
fn the_other_actions_keep_their_names() {
    // Act / Assert
    assert_eq!(parse(&["go-to-workspace", "3"]), Action::GoToWorkspace(3));
    assert_eq!(
        parse(&["send-to-workspace", "5"]),
        Action::SendToWorkspace(5)
    );
    assert_eq!(parse(&["focus", "left"]), Action::Focus(Direction::Left));
    assert_eq!(parse(&["move", "down"]), Action::Move(Direction::Down));
    assert_eq!(parse(&["cycle-layout"]), Action::CycleLayout);
}

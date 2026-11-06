use crate::config::Modifier::{Alt, Shift};
use crate::config::store::test_support::{TempStore, contents, temp_store};
use crate::config::{LaunchBinding, WindowMatch};

const FILE: &str = "# my keys\n[[keybinding]]\naction = \"focus-down\"\nkey = \"J\"\nmodifiers = [\"alt\"]\n\n# terminal\n[[launch]]\ncommand = \"wt.exe\" # windows terminal\nargs = [\"-p\", \"Dev\"]\nkey = \"T\"\nmodifiers = [\"alt\"]\ntheme = \"dark\"\n";

fn store_with(test: &str, text: &str) -> TempStore {
    let store = temp_store(test);
    std::fs::write(store.keybindings_path(), text).unwrap();
    store
}

fn app(command: &str, key: &str) -> LaunchBinding {
    LaunchBinding {
        command: command.into(),
        args: Vec::new(),
        key: key.into(),
        modifiers: vec![Alt],
        focus: false,
        windows: WindowMatch::default(),
    }
}

#[test]
fn the_launch_bindings_are_read_in_the_order_of_the_file() {
    // Arrange
    let text = format!("{FILE}\n[[launch]]\nkey = \"E\"\nmodifiers = []\n");
    let store = store_with("kb-launch-read", &text);

    // Act
    let launches = store.open_keybindings().unwrap().launches();

    // Assert -- the second has no command and stays at its place.
    assert_eq!(launches.len(), 2);
    assert_eq!(launches[0].as_ref().unwrap().args, ["-p", "Dev"]);
    assert!(launches[1].is_err());
}

#[test]
fn editing_a_launch_binding_keeps_its_comments_and_unknown_keys() {
    // Arrange
    let store = store_with("kb-launch-edit", FILE);
    let mut doc = store.open_keybindings().unwrap();
    let mut edited = doc.launches()[0].as_ref().unwrap().clone();
    edited.command = "pwsh.exe".into();
    edited.args.clear();
    edited.focus = true;
    edited.windows.match_title = Some("PowerShell".into());

    // Act
    doc.set_launch(0, &edited).unwrap();
    doc.set_launch_key(0, "p", &[Alt, Shift]).unwrap();
    doc.save().unwrap();

    // Assert
    let expected = FILE
        .replace("\"wt.exe\" # windows", "\"pwsh.exe\" # windows")
        .replace("args = [\"-p\", \"Dev\"]\n", "")
        .replace(
            "\"T\"\nmodifiers = [\"alt\"]",
            "\"P\"\nmodifiers = [\"alt\", \"shift\"]",
        )
        + "focus = true\nmatch_title = \"PowerShell\"\n";
    assert_eq!(contents(&store.keybindings_path()), expected);
}

#[test]
fn adding_and_removing_launch_bindings_leave_the_rest_alone() {
    // Arrange
    let store = store_with("kb-launch-add", FILE);
    let mut doc = store.open_keybindings().unwrap();

    // Act
    doc.add_launch(&app("notepad.exe", "N")).unwrap();
    doc.remove_launch(0).unwrap();
    doc.save().unwrap();

    // Assert
    let expected = "# my keys\n[[keybinding]]\naction = \"focus-down\"\nkey = \"J\"\nmodifiers = [\"alt\"]\n\n[[launch]]\ncommand = \"notepad.exe\"\nkey = \"N\"\nmodifiers = [\"alt\"]\n";
    assert_eq!(contents(&store.keybindings_path()), expected);
}

#[test]
fn a_launch_binding_on_the_key_of_a_keybinding_is_a_conflict() {
    // Arrange
    let store = store_with("kb-launch-clash", FILE);
    let mut doc = store.open_keybindings().unwrap();

    // Act
    doc.set_launch_key(0, "J", &[Alt]).unwrap();

    // Assert
    let conflicts = doc.conflicts();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].bindings, [0]);
    assert_eq!(conflicts[0].launches, [0]);
}

#[test]
fn a_launch_binding_without_a_command_or_a_shortcut_is_not_saved() {
    // Arrange
    let store = store_with("kb-launch-blank", FILE);
    let mut doc = store.open_keybindings().unwrap();

    // Act
    doc.add_launch(&app("calc.exe", "")).unwrap();
    let no_key = doc.save();
    doc.set_launch_key(1, "C", &[Alt]).unwrap();
    doc.set_launch(1, &app(" ", "C")).unwrap();
    let no_command = doc.save();

    // Assert
    let no_key = no_key.unwrap_err().to_string();
    assert!(
        no_key.contains("launch binding 2 (calc.exe) has no shortcut"),
        "{no_key}"
    );
    let no_command = no_command.unwrap_err().to_string();
    assert!(
        no_command.contains("launch binding 2 has no command"),
        "{no_command}"
    );
    assert_eq!(contents(&store.keybindings_path()), FILE);
}

#[test]
fn a_key_that_is_not_a_key_is_refused() {
    let store = store_with("kb-launch-badkey", FILE);
    let mut doc = store.open_keybindings().unwrap();

    let set = doc.set_launch_key(0, "NotAKey", &[Alt]);

    assert!(set.is_err());
    assert!(!doc.is_dirty());
}

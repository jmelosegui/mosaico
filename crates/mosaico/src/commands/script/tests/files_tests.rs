use std::fs;

use crate::commands::context::Exit;
use crate::commands::script::files;
use crate::commands::test_support::Harness;

#[test]
fn new_writes_the_template_and_the_editor_support_and_opens_it() {
    // Arrange
    let mut h = Harness::new("script-new");
    let dir = h.config.dir().to_path_buf();

    // Act
    let exit = h.run(|cx| files::new(cx, "meeting-mode"));

    // Assert
    assert_eq!(exit, Exit::Success);
    let script = dir.join("scripts").join("meeting-mode.lua");
    let text = fs::read_to_string(&script).unwrap();
    assert!(text.contains("mosaico script run meeting-mode"), "{text}");
    assert!(dir.join("lib").join("mosaico.lua").exists());
    assert!(dir.join(".luarc.json").exists());
    assert_eq!(h.world.opened(), vec![script.clone()]);
    assert_eq!(h.out(), format!("Created {}\n", script.display()));
}

#[test]
fn new_keeps_annotations_the_user_already_has() {
    // Arrange
    let mut h = Harness::new("script-new-keeps");
    let lib = h.config.dir().join("lib");
    fs::create_dir_all(&lib).unwrap();
    fs::write(lib.join("mosaico.lua"), "-- mine").unwrap();

    // Act
    h.run(|cx| files::new(cx, "a"));

    // Assert
    assert_eq!(
        fs::read_to_string(lib.join("mosaico.lua")).unwrap(),
        "-- mine"
    );
}

#[test]
fn new_refuses_a_bad_name_and_one_that_exists() {
    // Arrange
    let mut h = Harness::new("script-new-refuses");
    h.run(|cx| files::new(cx, "dock"));

    // Act
    let bad = h.run(|cx| files::new(cx, "my script"));
    let again = h.run(|cx| files::new(cx, "dock"));

    // Assert
    assert_eq!((bad, again), (Exit::Failure, Exit::Failure));
    assert!(
        h.err().contains("`my script` is not a script name"),
        "{}",
        h.err()
    );
    assert!(h.err().contains("mosaico script edit dock"), "{}", h.err());
}

#[test]
fn edit_opens_a_script_that_exists() {
    // Arrange
    let mut h = Harness::new("script-edit");
    h.run(|cx| files::new(cx, "dock"));

    // Act
    let found = h.run(|cx| files::edit(cx, "dock"));
    let missing = h.run(|cx| files::edit(cx, "nope"));

    // Assert
    assert_eq!((found, missing), (Exit::Success, Exit::Failure));
    assert_eq!(h.world.opened().len(), 2);
    assert!(h.err().contains("mosaico script new nope"), "{}", h.err());
}

#[test]
fn list_prints_the_names_in_order() {
    // Arrange
    let mut h = Harness::new("script-list");
    let empty = h.run(files::list);
    let scripts = h.config.dir().join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    for file in ["zeta.lua", "alpha.lua", "notes.txt"] {
        fs::write(scripts.join(file), "").unwrap();
    }

    // Act
    let listed = h.run(files::list);

    // Assert
    assert_eq!((empty, listed), (Exit::Success, Exit::Success));
    let out = h.out();
    assert!(out.starts_with("No scripts in"), "{out}");
    assert!(out.ends_with("\nalpha\nzeta\n"), "{out}");
}

#[test]
fn add_copies_a_script_in_under_its_name() {
    // Arrange
    let mut h = Harness::new("script-add");
    let source = h.config.dir().with_file_name("shared-dock.lua");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "print('dock')").unwrap();

    // Act
    let added = h.run(|cx| files::add(cx, &source));
    let again = h.run(|cx| files::add(cx, &source));

    // Assert
    assert_eq!((added, again), (Exit::Success, Exit::Failure));
    let copied = h.config.dir().join("scripts").join("shared-dock.lua");
    assert_eq!(fs::read_to_string(copied).unwrap(), "print('dock')");
}

#[test]
fn open_creates_the_scripts_folder_and_shows_it() {
    // Arrange
    let mut h = Harness::new("script-open");
    let scripts = h.config.dir().join("scripts");

    // Act
    let exit = h.run(files::open_folder);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert!(scripts.is_dir());
    assert_eq!(h.world.opened(), vec![scripts]);
}

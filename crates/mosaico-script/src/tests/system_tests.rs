//! Programs, output, JSON, arguments and what scripts may not reach.

use std::fs;
use std::time::Duration;

use super::support::World;
use crate::{Options, Outcome};

#[test]
fn launch_starts_the_program_with_its_arguments() {
    // Arrange
    let world = World::new();

    // Act
    world.run(r#"print(mosaico.launch{ "wt.exe", "-d", "C:/src" })"#);

    // Assert
    assert_eq!(world.out(), "true\n");
    assert_eq!(
        *world.launcher.0.borrow(),
        vec![vec!["wt.exe".to_string(), "-d".into(), "C:/src".into()]]
    );
}

#[test]
fn exec_runs_a_program_and_returns_what_it_did() {
    // Arrange
    let world = World::new();

    // Act
    world.run(
        r#"
        local r = mosaico.exec{ "git", "status", timeout = 5 }
        print(r.ok, r.code, r.stdout, r.timed_out)
        "#,
    );

    // Assert
    assert_eq!(world.out(), "true\t0\tran git status\n\tfalse\n");
    assert_eq!(
        *world.runner.0.borrow(),
        vec![(
            vec!["git".to_string(), "status".into()],
            Some(Duration::from_secs(5))
        )]
    );
}

#[test]
fn log_writes_to_the_error_stream_with_its_level() {
    // Arrange
    let world = World::new();

    // Act
    world.run(r#"mosaico.log("warn", "careful") mosaico.log("hello")"#);

    // Assert
    assert_eq!(world.err(), "[warn] careful\n[info] hello\n");
}

#[test]
fn json_goes_both_ways() {
    // Arrange
    let world = World::new();

    // Act
    world.run(
        r#"
        local t = mosaico.json.decode('{"a":[1,2],"b":null}')
        print(#t.a, t.b, mosaico.json.encode({ n = 1 }))
        "#,
    );

    // Assert
    assert_eq!(world.out(), "2\tnil\t{\"n\":1}\n");
}

#[test]
fn arguments_read_as_numbers_booleans_or_text() {
    // Arrange
    let world = World::new();
    let options = Options {
        args: vec![
            ("n".into(), "2".into()),
            ("on".into(), "true".into()),
            ("name".into(), "x".into()),
        ],
        ..Options::default()
    };

    // Act
    world
        .script_with(
            r#"print(mosaico.args.n + 1, mosaico.args.on, mosaico.args.name)"#,
            options,
        )
        .run();

    // Assert
    assert_eq!(world.out(), "3\ttrue\tx\n");
}

#[test]
fn a_shell_and_native_modules_are_out_of_reach() {
    // Arrange
    let world = World::new();

    // Act
    world.run(r#"print(os.execute, io.popen, os.exit, package.loadlib, package.cpath)"#);

    // Assert
    assert_eq!(world.out(), "nil\tnil\tnil\tnil\t\n");
}

#[test]
fn require_finds_modules_only_in_the_library_folder() {
    // Arrange
    let lib = std::env::temp_dir().join(format!("mosaico-script-lib-{}", std::process::id()));
    fs::create_dir_all(&lib).unwrap();
    fs::write(
        lib.join("util.lua"),
        "return { twice = function(n) return n * 2 end }",
    )
    .unwrap();
    let world = World::new();
    let options = Options {
        lib: Some(lib.clone()),
        ..Options::default()
    };

    // Act
    let found = world
        .script_with(r#"print(require("util").twice(21))"#, options.clone())
        .run();
    let missing = world.script_with(r#"require("nothere")"#, options).run();
    fs::remove_dir_all(&lib).unwrap();

    // Assert
    assert_eq!(found, Outcome::Finished);
    assert_eq!(world.out(), "42\n");
    assert!(matches!(missing, Outcome::Failed(m) if m.contains("nothere")));
}

#[test]
fn a_syntax_error_fails_with_the_script_name() {
    // Arrange
    let world = World::new();

    // Act
    let outcome = world.run("this is not lua");

    // Assert
    assert!(matches!(outcome, Outcome::Failed(m) if m.contains("test")));
}

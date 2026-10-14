use crate::config::Config;
use crate::config::store::test_support::{cleanup, unique_temp_path};

use super::*;

#[test]
fn migrates_legacy_section_and_writes_backup() {
    let path = unique_temp_path();
    let original = "\
[layout]
gap = 8

[layout.workspaces]
1 = \"vertical-stack\"
3 = \"three-column\"
";
    std::fs::write(&path, original).unwrap();

    let migrated = migrate_config_if_needed(&path, original).unwrap();

    assert!(migrated.is_some(), "expected migration to run");
    let new_content = migrated.unwrap();
    assert!(new_content.contains("[workspaces.layouts]"));
    assert!(!new_content.contains("[layout.workspaces]"));
    assert!(new_content.contains("1 = \"vertical-stack\""));

    let backup = path.with_extension("toml.pre-0.9.bak");
    assert!(backup.exists(), "backup file should exist");
    let backup_contents = std::fs::read_to_string(&backup).unwrap();
    assert_eq!(backup_contents, original, "backup must match original");

    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        on_disk, new_content,
        "rewritten file must match returned content"
    );

    cleanup(&path);
}

#[test]
fn no_legacy_section_is_a_noop() {
    let path = unique_temp_path();
    let original = "\
[layout]
gap = 8

[workspaces]
mode = \"global\"

[workspaces.layouts]
2 = \"bsp\"
";
    std::fs::write(&path, original).unwrap();

    let migrated = migrate_config_if_needed(&path, original).unwrap();
    assert!(
        migrated.is_none(),
        "should not migrate when no legacy section"
    );

    let backup = path.with_extension("toml.pre-0.9.bak");
    assert!(!backup.exists(), "no backup should be created");

    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert_eq!(on_disk, original, "file must be untouched");

    cleanup(&path);
}

#[test]
fn second_run_is_idempotent() {
    let path = unique_temp_path();
    let original = "\
[layout]
gap = 8

[layout.workspaces]
1 = \"bsp\"
";
    std::fs::write(&path, original).unwrap();

    // First pass migrates.
    let first = migrate_config_if_needed(&path, original).unwrap();
    assert!(first.is_some());
    let after_first = std::fs::read_to_string(&path).unwrap();

    // Second pass sees the already-migrated file and does nothing.
    let second = migrate_config_if_needed(&path, &after_first).unwrap();
    assert!(second.is_none(), "second migration must be a no-op");

    let collision_backup = path.with_extension("toml.pre-0.9.bak.1");
    assert!(
        !collision_backup.exists(),
        "no second backup should be created"
    );

    cleanup(&path);
}

#[test]
fn existing_workspaces_layouts_wins_over_legacy() {
    let path = unique_temp_path();
    let original = "\
[layout]
gap = 8

[layout.workspaces]
1 = \"bsp\"

[workspaces]
mode = \"global\"

[workspaces.layouts]
2 = \"three-column\"
";
    std::fs::write(&path, original).unwrap();

    let migrated = migrate_config_if_needed(&path, original).unwrap().unwrap();

    // Legacy section is gone.
    assert!(!migrated.contains("[layout.workspaces]"));
    // The pre-existing [workspaces.layouts] is preserved (workspace 2).
    assert!(migrated.contains("2 = \"three-column\""));
    // The legacy mapping (workspace 1 = bsp) is dropped because the
    // user already had a [workspaces.layouts] section.
    assert!(!migrated.contains("1 = \"bsp\""));

    cleanup(&path);
}

#[test]
fn backup_path_avoids_collision() {
    let path = unique_temp_path();
    std::fs::write(&path, "").unwrap();

    let first = backup_path(&path);
    assert_eq!(first.extension().and_then(|e| e.to_str()), Some("bak"));
    std::fs::write(&first, "").unwrap();

    let second = backup_path(&path);
    assert_ne!(first, second);
    assert!(
        second.to_string_lossy().ends_with(".bak.1"),
        "second backup should append .1 suffix, got {}",
        second.display()
    );

    cleanup(&path);
}

#[test]
fn migrates_legacy_border_colors() {
    let path = unique_temp_path();
    let original = "\
[borders]
width = 4
focused = \"#ff0000\"
monocle = \"#00ff00\"
";
    std::fs::write(&path, original).unwrap();

    let migrated = migrate_config_if_needed(&path, original)
        .unwrap()
        .expect("migration should run");

    // Re-parse and assert the values landed in the new shape.
    let cfg: Config = toml::from_str(&migrated).unwrap();
    assert_eq!(cfg.borders.width, 4);
    assert_eq!(cfg.borders.colors.focused, "#ff0000");
    assert_eq!(cfg.borders.colors.monocle, "#00ff00");

    // Backup preserved.
    let backup = path.with_extension("toml.pre-0.9.bak");
    assert!(backup.exists());
    assert_eq!(std::fs::read_to_string(&backup).unwrap(), original);

    cleanup(&path);
}

#[test]
fn border_migration_is_idempotent() {
    let path = unique_temp_path();
    let original = "\
[borders]
width = 4
focused = \"blue\"
";
    std::fs::write(&path, original).unwrap();

    migrate_config_if_needed(&path, original).unwrap().unwrap();
    let after_first = std::fs::read_to_string(&path).unwrap();
    let second = migrate_config_if_needed(&path, &after_first).unwrap();
    assert!(second.is_none(), "second border migration must be a no-op");

    cleanup(&path);
}

#[test]
fn existing_border_colors_wins_over_legacy() {
    let path = unique_temp_path();
    let original = "\
[borders]
width = 4
focused = \"red\"
monocle = \"green\"

[borders.colors]
focused = \"blue\"
";
    std::fs::write(&path, original).unwrap();

    let migrated = migrate_config_if_needed(&path, original).unwrap().unwrap();
    let cfg: Config = toml::from_str(&migrated).unwrap();

    // Existing nested focused wins over legacy.
    assert_eq!(cfg.borders.colors.focused, "blue");
    // Legacy monocle migrates because no nested value existed.
    assert_eq!(cfg.borders.colors.monocle, "green");

    cleanup(&path);
}

#[test]
fn workspace_and_border_migrations_run_together() {
    let path = unique_temp_path();
    let original = "\
[layout]
gap = 8

[layout.workspaces]
1 = \"vertical-stack\"

[borders]
width = 4
focused = \"#abcdef\"
";
    std::fs::write(&path, original).unwrap();

    let migrated = migrate_config_if_needed(&path, original).unwrap().unwrap();

    // Both legacy patterns are gone after migration.
    assert!(!migrated.contains("[layout.workspaces]"));
    assert!(migrated.contains("[workspaces.layouts]"));
    assert!(migrated.contains("[borders.colors]"));
    // Top-level `focused` under [borders] should not be there anymore;
    // the only remaining `focused = ...` line lives under [borders.colors].
    let bare_focused_lines = migrated
        .lines()
        .filter(|l| l.trim_start().starts_with("focused = "))
        .count();
    assert_eq!(
        bare_focused_lines, 1,
        "exactly one focused = line, under [borders.colors]"
    );

    // Only one backup created for the combined run.
    let backup = path.with_extension("toml.pre-0.9.bak");
    let collision = path.with_extension("toml.pre-0.9.bak.1");
    assert!(backup.exists());
    assert!(!collision.exists(), "single backup, not one per migration");

    cleanup(&path);
}

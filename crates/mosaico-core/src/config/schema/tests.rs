use super::*;
use crate::action::MAX_WORKSPACES;
use crate::config::Config;

/// The keys of the single values in `table`, joined with dots, under
/// `prefix`.
fn leaf_keys(prefix: &str, table: &toml::Table, keys: &mut Vec<String>) {
    for (name, value) in table {
        let key = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        match value {
            toml::Value::Table(inner) => leaf_keys(&key, inner, keys),
            _ => keys.push(key),
        }
    }
}

#[test]
fn every_setting_of_the_default_config_is_described_once() {
    // Arrange
    let defaults = toml::Table::try_from(Config::default()).unwrap();
    let mut keys = Vec::new();
    leaf_keys("", &defaults, &mut keys);
    // The layouts of the workspaces are a table the default config leaves empty.
    keys.extend((1..=MAX_WORKSPACES).map(|n| format!("workspaces.layouts.{n}")));
    keys.sort();

    // Act
    let mut described: Vec<String> = CONFIG_SETTINGS.iter().map(|s| s.key.into()).collect();
    described.sort();

    // Assert
    assert_eq!(described, keys);
}

#[test]
fn every_default_is_a_value_its_setting_takes() {
    for setting in CONFIG_SETTINGS {
        let default = config_default(setting.key).expect(setting.key);
        assert_eq!(setting.check(&default), Ok(()), "{}", setting.key);
    }
}

#[test]
fn a_number_out_of_range_is_brought_into_it_as_loading_does() {
    // The log size is only kept in range by the settings window.
    let clamped_on_load = CONFIG_SETTINGS
        .iter()
        .filter(|s| matches!(s.kind, SettingKind::Int { .. } | SettingKind::Float { .. }))
        .filter(|s| s.key != "logging.max_file_mb");
    for setting in clamped_on_load {
        for raw in [-100_000, 100_000] {
            // Arrange
            let (tables, name) = setting.path();
            let text = format!("[{}]\n{name} = {raw}\n", tables.join("."));
            let mut config: Config = toml::from_str(&text).expect(&text);
            config.validate();
            let loaded = toml::Table::try_from(config).unwrap();
            let mut value = toml::Value::Table(loaded);
            for part in setting.key.split('.') {
                value = value.as_table().unwrap()[part].clone();
            }

            // Act
            let normalized = setting.normalize(SettingValue::Int(raw));

            // Assert
            let expected = SettingValue::from_toml(&value).unwrap();
            assert_eq!(normalized, Some(expected), "{} = {raw}", setting.key);
        }
    }
}

#[test]
fn a_value_of_another_kind_or_an_unlisted_name_is_refused() {
    let hiding = find("layout.hiding").unwrap();
    let gap = find("layout.gap").unwrap();

    assert!(hiding.check(&SettingValue::Text("hide".into())).is_ok());
    assert!(hiding.check(&SettingValue::Text("vanish".into())).is_err());
    assert!(gap.check(&SettingValue::Bool(true)).is_err());
    assert!(gap.check(&SettingValue::Int(201)).is_err());
    assert_eq!(hiding.normalize(SettingValue::Text("vanish".into())), None);
}

#[test]
fn only_the_unfocused_border_color_can_be_none() {
    let none: Vec<_> = CONFIG_SETTINGS
        .iter()
        .filter(|s| s.allows_none())
        .map(|s| s.key)
        .collect();

    assert_eq!(none, ["borders.colors.unfocused"]);
}

#[test]
fn a_setting_is_split_into_its_tables_and_its_name() {
    let focused = find("borders.colors.focused").unwrap();
    assert_eq!(focused.path(), (vec!["borders", "colors"], "focused"));
    assert_eq!(find("layout.nothing"), None);
}

#[test]
fn a_workspace_has_no_layout_of_its_own_by_default() {
    assert_eq!(
        config_default("workspaces.layouts.3"),
        Some(SettingValue::Text(String::new()))
    );
    assert_eq!(config_default("workspaces.layouts.9"), None);
}

mod bar {
    use super::*;
    use crate::config::BarConfig;

    /// Lists, which the settings window edits on their own.
    const LISTS: [&str; 4] = ["monitors", "mouse.click", "left", "right"];

    #[test]
    fn every_setting_of_the_default_bar_is_described_once() {
        // Arrange
        let defaults = toml::Table::try_from(BarConfig::default()).unwrap();
        let mut keys = Vec::new();
        leaf_keys("", &defaults, &mut keys);
        keys.retain(|k| !LISTS.contains(&k.as_str()));
        keys.sort();

        // Act
        let mut described: Vec<String> = BAR_SETTINGS.iter().map(|s| s.key.into()).collect();
        described.sort();

        // Assert
        assert_eq!(described, keys);
    }

    #[test]
    fn every_default_is_a_value_its_setting_takes() {
        for setting in BAR_SETTINGS {
            let default = bar_default(setting.key).expect(setting.key);
            assert_eq!(setting.check(&default), Ok(()), "{}", setting.key);
        }
    }

    #[test]
    fn a_number_out_of_range_is_brought_into_it_as_loading_does() {
        let numbers = BAR_SETTINGS
            .iter()
            .filter(|s| matches!(s.kind, SettingKind::Int { .. }));
        for setting in numbers {
            for raw in [-100_000, 100_000] {
                // Arrange
                let (tables, name) = setting.path();
                let header = if tables.is_empty() {
                    String::new()
                } else {
                    format!("[{}]\n", tables.join("."))
                };
                let text = format!("{header}{name} = {raw}\n");
                let mut config: BarConfig = toml::from_str(&text).expect(&text);
                config.validate();
                let mut value = toml::Value::try_from(config).unwrap();
                for part in setting.key.split('.') {
                    value = value.as_table().unwrap()[part].clone();
                }

                // Act
                let normalized = setting.normalize(SettingValue::Int(raw));

                // Assert
                let expected = SettingValue::from_toml(&value).unwrap();
                assert_eq!(normalized, Some(expected), "{} = {raw}", setting.key);
            }
        }
    }

    #[test]
    fn the_settings_are_found_by_their_keys() {
        assert!(find_bar("colors.pill_border").is_some());
        assert!(find_bar("layout.gap").is_none());
    }
}

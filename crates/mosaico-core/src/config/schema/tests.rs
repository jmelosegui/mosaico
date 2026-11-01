use super::*;
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
fn a_setting_is_split_into_its_tables_and_its_name() {
    let focused = find("borders.colors.focused").unwrap();
    assert_eq!(focused.path(), (vec!["borders", "colors"], "focused"));
    assert_eq!(find("layout.nothing"), None);
}

use config_parser::{Config, Mode, parse_config};
use serde_json::json;

#[test]
fn test_parse_valid_config() {
    let json_data = json!({
        "name": "MyApp",
        "version": "1.0.0",
        "settings": {
            "theme": "dark",
            "max_connections": 100
        },
        "features": ["feature1", "feature2"],
        "mode": "development"
    })
    .to_string();

    let config = parse_config(&json_data).expect("Failed to parse valid config");
    assert_eq!(config.name, "MyApp");
    assert_eq!(config.version, "1.0.0");
    assert_eq!(config.settings.theme.unwrap(), "dark");
    assert_eq!(config.settings.max_connections.unwrap(), 100);
    assert_eq!(config.features, vec!["feature1", "feature2"]);
    assert_eq!(config.mode, Mode::Development);
}

#[test]
fn test_parse_invalid_config() {
    let json_data = json!({
        "name": "",
        "version": "",
        "settings": {},
        "features": [],
        "mode": "development"
    })
    .to_string();

    let result = parse_config(&json_data);
    assert!(result.is_err(), "Invalid config should return an error");
}

#[test]
fn test_case_insensitive_mode() {
    let json_data = json!({
        "name": "MyApp",
        "version": "1.0.0",
        "settings": {
            "theme": "light",
            "max_connections": 50
        },
        "features": ["featureA", "featureB"],
        "mode": "DeVeLoPmEnT"
    })
    .to_string();

    let config = parse_config(&json_data).expect("Failed to parse config with case-insensitive mode");
    assert_eq!(config.mode, Mode::Development);
}

#[test]
fn test_default_settings() {
    let json_data = json!({
        "name": "DefaultApp",
        "version": "2.0.0",
        "features": ["default_feature"],
        "mode": "production"
    })
    .to_string();

    let config = parse_config(&json_data).expect("Failed to parse config with default settings");
    assert_eq!(config.settings.theme.unwrap(), "default");
    assert_eq!(config.settings.max_connections.unwrap(), 10);
}

use cage::config::ConfigError;

#[test]
fn test_config_error_display() {
    let error = ConfigError::Read(std::io::Error::new(std::io::ErrorKind::Other, "test"));
    assert_eq!(format!("{}", error), "failed to read config: test");

    // construct a real toml parse error by attempting to parse invalid TOML
    let toml_err = toml::from_str::<toml::Value>("not = valid =").unwrap_err();
    let error = ConfigError::Parse(toml_err);
    let msg = format!("{}", error);
    assert!(msg.starts_with("failed to parse config:"));
}

use std::fs;
use std::path::PathBuf;

use cage::config::{CONFIG_FILENAME, Config, ConfigError, DEFAULT_CAGERC, Network};
use tempfile::tempdir;

#[test]
fn init_creates_config_when_absent() {
    let dir = tempdir().unwrap();
    Config::init_in(dir.path(), false).unwrap();

    assert!(dir.path().join(CONFIG_FILENAME).exists());
}

#[test]
fn init_does_not_overwrite_existing_config_without_force() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(CONFIG_FILENAME);
    let custom = "[cage]\nnetwork = \"permit\"\n";
    fs::write(&config_path, custom).unwrap();

    let config = Config::init_in(dir.path(), false).unwrap();

    assert_eq!(config.cage.network, Network::Permit);
    assert_eq!(fs::read_to_string(&config_path).unwrap(), custom);
}

#[test]
fn init_overwrites_existing_config_with_force() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(CONFIG_FILENAME);
    fs::write(&config_path, "[cage]\nnetwork = \"permit\"\n").unwrap();

    let config = Config::init_in(dir.path(), true).unwrap();

    assert_eq!(config.cage.network, Network::Deny);
    assert_eq!(fs::read_to_string(&config_path).unwrap(), DEFAULT_CAGERC);
}

#[test]
fn init_returns_parse_error_for_invalid_toml() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(CONFIG_FILENAME);
    fs::write(&config_path, "not = valid = toml").unwrap();

    let result = Config::init_in(dir.path(), false);

    assert!(matches!(result, Err(ConfigError::Parse(_))));
}

#[test]
fn init_returns_parse_error_when_cage_section_missing() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(CONFIG_FILENAME);
    fs::write(&config_path, "[filesystem]\nro = []\n").unwrap();

    let result = Config::init_in(dir.path(), false);

    assert!(matches!(result, Err(ConfigError::Parse(_))));
}

#[test]
fn network_deserializes_lowercase_variants() {
    let deny: Config = toml::from_str("[cage]\nnetwork = \"deny\"").unwrap();
    let permit: Config = toml::from_str("[cage]\nnetwork = \"permit\"").unwrap();

    assert_eq!(deny.cage.network, Network::Deny);
    assert_eq!(permit.cage.network, Network::Permit);
}

#[test]
fn network_rejects_non_lowercase_variant() {
    let result = toml::from_str::<Config>("[cage]\nnetwork = \"Deny\"");

    assert!(result.is_err());
}

#[test]
fn network_rejects_unknown_variant() {
    let result = toml::from_str::<Config>("[cage]\nnetwork = \"maybe\"");

    assert!(result.is_err());
}

#[test]
fn optional_sections_default_to_empty_when_omitted() {
    let config: Config = toml::from_str("[cage]\nnetwork = \"deny\"").unwrap();

    assert!(config.filesystem.ro.is_empty());
    assert!(config.filesystem.rw.is_empty());
    assert!(config.env.unset.is_empty());
    assert!(config.internal.expose.is_empty());
    assert!(config.external.allow.is_empty());
    assert!(config.agents.is_empty());
}

#[test]
fn parses_full_config_with_all_sections() {
    let toml_str = r#"
[cage]
network = "permit"

[filesystem]
ro = ["/usr", "/bin"]
rw = ["."]

[env]
unset = ["SSH_AUTH_SOCK"]

[internal]
expose = ["127.0.0.1:5432"]

[external]
allow = ["example.com:443"]

[agents.claude]
allow = ["api.anthropic.com:443"]
pass = ["ANTHROPIC_API_KEY"]
rw = ["/tmp"]
expose = ["8080"]
"#;

    let config: Config = toml::from_str(toml_str).unwrap();

    assert_eq!(config.cage.network, Network::Permit);
    assert_eq!(
        config.filesystem.ro,
        vec![PathBuf::from("/usr"), PathBuf::from("/bin")]
    );
    assert_eq!(config.filesystem.rw, vec![PathBuf::from(".")]);
    assert_eq!(config.env.unset, vec!["SSH_AUTH_SOCK".to_string()]);
    assert_eq!(config.internal.expose, vec!["127.0.0.1:5432".to_string()]);
    assert_eq!(config.external.allow, vec!["example.com:443".to_string()]);

    let claude = config.agents.get("claude").unwrap();
    assert_eq!(claude.allow, vec!["api.anthropic.com:443".to_string()]);
    assert_eq!(claude.pass, vec!["ANTHROPIC_API_KEY".to_string()]);
    assert_eq!(claude.rw, vec![PathBuf::from("/tmp")]);
    assert_eq!(claude.expose, vec!["8080".to_string()]);
}

#[test]
fn parses_multiple_agent_profiles() {
    let toml_str = r#"
[cage]
network = "deny"

[agents.claude]
allow = ["api.anthropic.com:443"]

[agents.codex]
allow = ["api.openai.com:443"]
"#;

    let config: Config = toml::from_str(toml_str).unwrap();

    assert_eq!(config.agents.len(), 2);
    assert!(config.agents.contains_key("claude"));
    assert!(config.agents.contains_key("codex"));
}

#[test]
fn agent_profile_fields_default_to_empty_when_omitted() {
    let toml_str = r#"
[cage]
network = "deny"

[agents.minimal]
"#;

    let config: Config = toml::from_str(toml_str).unwrap();
    let profile = config.agents.get("minimal").unwrap();

    assert!(profile.allow.is_empty());
    assert!(profile.pass.is_empty());
    assert!(profile.rw.is_empty());
    assert!(profile.expose.is_empty());
}

#[test]
fn default_config_template_parses_with_expected_values() {
    let dir = tempdir().unwrap();
    let config = Config::init_in(dir.path(), false).unwrap();

    assert_eq!(config.cage.network, Network::Deny);
    assert_eq!(
        config.filesystem.ro,
        vec![
            PathBuf::from("/usr"),
            PathBuf::from("/bin"),
            PathBuf::from("/lib")
        ]
    );
    assert_eq!(config.filesystem.rw, vec![PathBuf::from(".")]);
    assert_eq!(config.env.unset, vec!["SSH_AUTH_SOCK".to_string()]);
    assert!(config.internal.expose.is_empty());
}

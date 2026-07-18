use core::fmt;
use std::io;

#[derive(Debug)]
pub enum ConfigError {
    Read(io::Error),
    Parse(toml::de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Parse(e) => write!(f, "failed to parse config: {e}"),
            ConfigError::Read(e) => write!(f, "failed to read config: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::Read(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::Parse(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

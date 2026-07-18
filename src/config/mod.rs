pub mod error;
pub mod template;

use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

pub use error::ConfigError;
pub use template::{CONFIG_FILENAME, DEFAULT_CAGERC};

#[derive(Deserialize)]
pub struct Config {
    pub cage: Cage,
    #[serde(default)]
    pub filesystem: Filesystem,
    #[serde(default)]
    pub env: Env,
    #[serde(default)]
    pub internal: Internal,
    #[serde(default)]
    pub external: External,
    #[serde(default)]
    pub agents: HashMap<String, Profile>,
}

#[derive(Deserialize)]
pub struct Cage {
    pub network: Network,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Deny,
    Permit,
}

#[derive(Deserialize, Default)]
pub struct Filesystem {
    #[serde(default)]
    pub ro: Vec<PathBuf>,
    #[serde(default)]
    pub rw: Vec<PathBuf>,
}

#[derive(Deserialize, Default)]
pub struct Env {
    #[serde(default)]
    pub unset: Vec<String>,
}

#[derive(Deserialize, Default)]
pub struct Internal {
    #[serde(default)]
    pub expose: Vec<String>,
}

#[derive(Deserialize, Default)]
pub struct External {
    #[serde(default)]
    pub allow: Vec<String>,
}

#[derive(Deserialize, Default)]
pub struct Profile {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub pass: Vec<String>,
    #[serde(default)]
    pub rw: Vec<PathBuf>,
    #[serde(default)]
    pub expose: Vec<String>,
}

impl Config {
    fn create(config_path: &Path) -> Result<(), ConfigError> {
        fs::write(config_path, DEFAULT_CAGERC)?;
        Ok(())
    }

    pub fn init_in(dir: &Path, force: bool) -> Result<Self, ConfigError> {
        let config_path = dir.join(CONFIG_FILENAME);

        if !config_path.exists() || force {
            Self::create(&config_path)?;
        }

        let config_content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&config_content)?;

        Ok(config)
    }

    fn init(force: bool) -> Result<Self, ConfigError> {
        let cwd = std::env::current_dir().map_err(ConfigError::Read)?;
        Self::init_in(&cwd, force)
    }
}


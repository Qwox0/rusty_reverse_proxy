use serde::{Deserialize, Serialize};
use std::{fs, io};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReverseProxyConfig {}

impl ReverseProxyConfig {
    pub fn new(cmd_arg: Option<String>) -> Result<ReverseProxyConfig, ConfigError> {
        Self::with_path(cmd_arg.unwrap_or_else(Self::default_config_file_path))
    }

    pub fn with_path(path: String) -> Result<ReverseProxyConfig, ConfigError> {
        let text = fs::read_to_string(path)?;

        panic!("{:?}", text.parse::<toml::Table>());

        toml::from_str(&text).map_err(Into::into)
    }

    pub fn default_config_file_path() -> String {
        "./revproxy.toml".to_string()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("")]
    NoConfigFile(), // TODO

    #[error("couldn't open config file: {}", .0)]
    OpenError(#[from] io::Error),

    #[error("couldn't deserialize config file: {}", .0)]
    ParseError(#[from] toml::de::Error),
}

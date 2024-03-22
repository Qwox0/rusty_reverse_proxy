use reqwest::header::ToStrError;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum ReverseProxyError {
    #[error(transparent)]
    ConfigError(#[from] ConfigError),

    #[error("couln't serve axum server: {}", .0)]
    AxumServeError(io::Error),

    #[error("request was missing the mandatory `host` request field")]
    MissingHostField,

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    RequestBodyTooLarge(axum::Error),

    #[error("non-ASCII char in request header field: {}", .0)]
    InvalidHeaderChar(ToStrError),
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

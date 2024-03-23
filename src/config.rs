use crate::{error::ConfigError, util::RequestExtract};
use axum::extract::{Host, Request};
use axum_server::tls_rustls::RustlsConfig;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fs, net::SocketAddr, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    pub host: Box<str>,
    #[serde(default)]
    pub path: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteConfig {
    pub request: Address,
    pub target: Address,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsConfig {
    fullchain_path: Box<Path>,
    privkey_path: Box<Path>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReverseProxyConfig {
    pub logging: bool,
    #[serde(
        serialize_with = "serialize_socket_addr",
        deserialize_with = "deserialize_socket_addr"
    )]
    pub address: SocketAddr,
    pub router_page: Option<Address>,
    #[serde(rename = "tls")]
    pub tls_config: Option<TlsConfig>,
    pub routes: Box<[RouteConfig]>,
}

impl Default for ReverseProxyConfig {
    fn default() -> Self {
        Self {
            logging: true,
            address: "[::]:8080".parse().unwrap(),
            router_page: None,
            tls_config: None,
            routes: Default::default(),
        }
    }
}

impl ReverseProxyConfig {
    pub fn new(cmd_arg: Option<String>) -> Result<ReverseProxyConfig, ConfigError> {
        Self::with_path(cmd_arg.unwrap_or_else(Self::default_config_file_path))
    }

    pub fn with_path(path: String) -> Result<ReverseProxyConfig, ConfigError> {
        let text = fs::read_to_string(path)?;

        toml::from_str(&text).map_err(Into::into)
    }

    pub fn default_config_file_path() -> String {
        "./revproxy.toml".to_string()
    }

    pub async fn tls(&self) -> Option<RustlsConfig> {
        let TlsConfig { fullchain_path, privkey_path } = self.tls_config.as_ref()?;

        Some(
            RustlsConfig::from_pem_chain_file(fullchain_path, privkey_path)
                .await
                .expect("can configure TLS"),
        )
    }

    pub fn leak(self) -> &'static Self {
        Box::leak(Box::new(self))
    }

    pub fn server_uses_tls(&self) -> bool {
        self.tls_config.is_some()
    }

    pub fn request_scheme(&self) -> &'static str {
        if self.server_uses_tls() { "https" } else { "http" }
    }
}

pub fn serialize_socket_addr<S>(addr: &SocketAddr, s: S) -> Result<S::Ok, S::Error>
where S: Serializer {
    addr.to_string().serialize(s)
}

pub fn deserialize_socket_addr<'de, D>(d: D) -> Result<SocketAddr, D::Error>
where D: Deserializer<'de> {
    <Box<str>>::deserialize(d)?.parse().map_err(serde::de::Error::custom)
}

impl Address {
    pub fn new(host: Box<str>, path: Box<str>) -> Address {
        Address { host, path }
    }

    pub async fn extract_from(req: Request) -> (Self, Request) {
        let (req, host) = req.extract::<Host>().await.unwrap();
        let path = req.uri().path();
        (Self::new(host.0.into(), path.into()), req)
    }

    /// `host` must be equal
    /// `path` must start with `other.path`
    pub fn matches(&self, other: &Address) -> bool {
        self.host == other.host && self.path.starts_with(other.path.as_ref())
    }
}

use crate::config::ReverseProxyConfig;
use reqwest::Client;

pub struct AppState {
    pub config: ReverseProxyConfig,
    pub reqwest_client: reqwest::Client,
}

impl AppState {
    pub fn new(config: ReverseProxyConfig) -> AppState {
        AppState { config, reqwest_client: Client::new() }
    }

    pub fn leak(self) -> &'static Self {
        Box::leak(Box::new(self))
    }
}

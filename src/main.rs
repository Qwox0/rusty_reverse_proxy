use axum::{
    extract::Request, http::StatusCode, middleware::Next, response::Response, routing::get, Router,
};
use rusty_reverse_proxy::config::{ConfigError, ReverseProxyConfig};

#[derive(Debug, thiserror::Error)]
pub enum ReverseProxyError {
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
}

async fn reverse_proxy(request: Request) -> () {
    println!("{:?}", request);
}

#[tokio::main]
async fn main() -> Result<(), ReverseProxyError> {
    // let arg1 = env::args().skip(1).next();
    // let config = ReverseProxyConfig::new(arg1)?;
    // println!("{:?}", config);
    let port = 8080;

    let app = Router::new().route("*path", axum::routing::any(reverse_proxy));

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

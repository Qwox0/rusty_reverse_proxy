use axum::{
    body::Body,
    extract::{ConnectInfo, FromRequestParts, Host, Request, State},
    http::{request::Parts, StatusCode},
    response::Response,
    Router,
};
use rusty_reverse_proxy::{
    app_state::AppState,
    config::{ReverseProxyConfig, RouteConfig},
    debug::{debugln, DebugBuf},
    error::ReverseProxyError,
};
use std::{env, net::SocketAddr};

pub async fn axum_request_to_reqwest(
    target_host: &str,
    request: Request<axum::body::Body>,
    state: &AppState,
) -> Result<reqwest::Request, ReverseProxyError> {
    let (Parts { method, uri, headers, version, .. }, body) = request.into_parts();
    let reqwest_body = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(ReverseProxyError::RequestBodyTooLarge)?;

    let url = format!("http://{target_host}{uri}");

    state
        .reqwest_client
        .request(method, url)
        .version(version)
        .headers(headers)
        .body(reqwest_body)
        .build()
        .map_err(Into::into)
}

pub fn reqwest_response_to_axum(res: reqwest::Response) -> axum::response::Response {
    let mut response = Response::new(());
    *response.status_mut() = res.status();
    *response.version_mut() = res.version();
    *response.headers_mut() = res.headers().clone();
    // *response.extensions_mut() = res.extensions().clone(); // ?
    let body = Body::from_stream(res.bytes_stream());
    let (parts, _) = response.into_parts();
    Response::from_parts(parts, body)
}

pub async fn request_send(
    request: Request<axum::body::Body>,
    target_host: &str,
    state: &AppState,
) -> Result<axum::response::Response, ReverseProxyError> {
    let (mut parts, body) = request.into_parts();
    let req_addr = ConnectInfo::<SocketAddr>::from_request_parts(&mut parts, &()).await.unwrap().0;

    let debug_msg = DebugBuf::new().display(req_addr).write_str(": ");
    let request = Request::from_parts(parts, body);
    let debug_msg = debug_msg.axum_req_with_scheme(&request, state.config.request_scheme());

    let (Parts { method, uri, headers, version, .. }, body) = request.into_parts();

    let reqwest_body = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(ReverseProxyError::RequestBodyTooLarge)?;

    let url = format!("http://{target_host}{uri}");

    let request = state
        .reqwest_client
        .request(method, url)
        .version(version)
        .headers(headers)
        .body(reqwest_body)
        .build()?;

    debug_msg.to().reqwest_req(&request).infoln();

    state
        .reqwest_client
        .execute(request)
        .await
        .map(reqwest_response_to_axum)
        .map_err(Into::into)
}

async fn reverse_proxy(
    State(state): State<&AppState>,
    request: Request,
) -> Result<Response, StatusCode> {
    debugln(&request);
    let (mut parts, body) = request.into_parts();
    let host = Host::from_request_parts(&mut parts, &()).await.unwrap();

    let request = Request::from_parts(parts, body);

    match state.config.routes.iter().find(|route| route.request.host.as_ref() == host.0) {
        Some(RouteConfig { target, .. }) => {
            request_send(request, target.host.as_ref(), state).await.map_err(|err| {
                DebugBuf::new().display(err).errorln();
                StatusCode::INTERNAL_SERVER_ERROR
            })
        },
        None => {
            DebugBuf::new()
                .axum_req_with_scheme(&request, state.config.request_scheme())
                .to()
                .not_found()
                .infoln();
            Err(StatusCode::NOT_FOUND)
        },
    }
}

async fn server() -> Result<(), ReverseProxyError> {
    let arg1 = env::args().skip(1).next();
    let config = ReverseProxyConfig::new(arg1)?;
    DebugBuf::new().display("Config: ").debug(&config).infoln();
    let state = AppState::new(config).leak();

    let router = Router::new();
    let router = router;
    let app = router
        .fallback(reverse_proxy)
        .with_state(state)
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = state.config.address;
    match state.config.tls().await {
        Some(tls_config) => {
            DebugBuf::new().display("listening on https://").display(addr).infoln();
            axum_server::bind_rustls(addr, tls_config).serve(app).await
        },
        None => {
            DebugBuf::new().display("listening on http://").display(addr).infoln();
            axum_server::bind(addr).serve(app).await
        },
    }
    .map_err(ReverseProxyError::AxumServeError)
}

#[tokio::main]
async fn main() {
    if let Err(err) = server().await {
        DebugBuf::new().display(err).errorln();
    }
}

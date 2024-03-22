use axum::{
    body::Body,
    extract::{FromRequest, FromRequestParts, Host, Path, Request, State},
    http::{request::Parts, StatusCode},
    response::Response,
    Router,
};
use rusty_reverse_proxy::{
    app_state::AppState,
    config::{ReverseProxyConfig, RouteConfig},
    debug::DebugBuf,
    error::ReverseProxyError,
    req,
};
use std::env;

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
    target_host: &str,
    request: Request<axum::body::Body>,
    state: &AppState,
) -> Result<axum::response::Response, ReverseProxyError> {
    let debug_msg = DebugBuf::new().axum_req_with_scheme(&request, state.config.request_scheme());

    let request = axum_request_to_reqwest(target_host, request, state).await?;

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
    // println!("{:?}", request);
    let (mut parts, body) = request.into_parts();
    let host = Host::from_request_parts(&mut parts, &()).await.unwrap();
    // println!("HOST: {:?}", host);
    // println!("PATH: {:?}", parts.uri);
    let request = Request::from_parts(parts, body);

    let route = state.config.routes.iter().find(|route| route.addr.host.as_ref() == host.0);

    match route {
        Some(RouteConfig { target, .. }) => {
            request_send(target.host.as_ref(), request, state).await.map_err(|err| {
                DebugBuf::new().val(err).errorln();
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
        }, // TODO
    }
}

#[tokio::main]
async fn main() -> Result<(), ReverseProxyError> {
    let arg1 = env::args().skip(1).next();
    let config = ReverseProxyConfig::new(arg1)?;
    println!("config: {:?}", config);
    let state = AppState::new(config).leak();

    let app = Router::new().fallback(reverse_proxy).with_state(state);

    let addr = state.config.address;
    match state.config.tls().await {
        Some(tls_config) => {
            println!("listening on https://{}", addr);
            axum_server::bind_rustls(addr, tls_config).serve(app.into_make_service()).await
        },
        None => {
            println!("listening on http://{}", addr);
            axum_server::bind(addr).serve(app.into_make_service()).await
        },
    }
    .map_err(ReverseProxyError::AxumServeError)
}

use crate::{
    app_state::AppState,
    config::{Address, ReverseProxyConfig, RouteConfig},
    debug::DebugBuf,
    util::RequestExtract,
};
use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
use std::net::SocketAddr;

pub async fn router_page(State(state): State<&AppState>, req: Request, next: Next) -> Response {
    let Some(router_page) = state.config.router_page.as_ref() else {
        return next.run(req).await;
    };
    let (addr, req) = Address::extract_from(req).await;
    if addr != *router_page {
        return next.run(req).await;
    }

    let (req, req_addr) = req.extract::<ConnectInfo<SocketAddr>>().await.unwrap();
    DebugBuf::new()
        .display(req_addr.0)
        .write_str(": ")
        .axum_req_with_scheme(&req, state.config.request_scheme())
        .to()
        .display("Reverse Proxy Router")
        .infoln();

    let scheme = state.config.request_scheme();
    let routes = state
        .config
        .routes
        .iter()
        .map(|route| format!("{scheme}://{}{}", route.request.host, route.request.path))
        .map(|url| format!("<li><a href=\"{url}\">{url}</a></li>\n",))
        .collect::<String>();

    Html(format!(
        r#"<!DOCTYPE HTML>
<html lang="en">

<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">

    <title>Reverse Proxy Router</title>
    <meta name="description" content="Lists all available routes managed by the reverse proxy.">
</head>

<body>
    <h1>Reverse Proxy Router</h1>
    <ul>
        {routes}
    </ul>
</body>

</html>"#
    ))
    .into_response()
}

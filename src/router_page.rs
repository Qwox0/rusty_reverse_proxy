use crate::{app_state::AppState, config::Address, debug::DebugBuf, util::RequestExtract};
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
    let router_page = "<h1>Reverse Proxy Router</h1>\n<ul>\n".to_string()
        + &state
            .config
            .routes
            .iter()
            .map(|route| format!("{scheme}://{}{}", route.request.host, route.request.path))
            .map(|url| format!("<li><a href=\"{url}\">{url}</a></li>\n",))
            .collect::<String>()
        + "</ul>";

    Html(router_page).into_response()
}

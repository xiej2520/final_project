use std::net::SocketAddr;

use axum::Router;

use tower::ServiceBuilder;

use tower_http::trace::TraceLayer;

use server::http_client::HttpClient;
use server::routers::*;
use server::CONFIG;
use server::{append_headers, init_logging, print_request_response};

/// Runs a search and reverse geoencoding (address) service
/// Reverse proxy traffic here *after* verifyinng authentication, this doesn't
/// do any authentication.
#[tokio::main]
async fn main() {
    init_logging();
    let search_client = HttpClient::new(CONFIG.search_url).unwrap();
    let address_client = HttpClient::new(CONFIG.search_url).unwrap();

    let mut search_app = Router::new()
        .nest(
            "/api",
            search_router::new_router().with_state(search_client.clone()),
        )
        .nest(
            "/api",
            address_router::new_router().with_state(address_client.clone()),
        )
        .layer(axum::middleware::from_fn(append_headers));

    if !cfg!(feature = "disable_logs") {
        search_app = search_app.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(print_request_response)),
        );
    }

    let addr = SocketAddr::from((CONFIG.ip, CONFIG.http_port));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::debug!("Server listening on {}", addr);
    axum::serve(listener, search_app).await.unwrap();
}

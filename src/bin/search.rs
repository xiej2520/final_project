use std::net::SocketAddr;

use axum::Router;

use tower::ServiceBuilder;

use tower_http::trace::TraceLayer;

use server::http_client::HttpClient;
use server::routers::{address_router, search_router};
use server::CONFIG;
use server::{init_logging, print_request_response};

/// Runs a search and reverse geoencoding (address) service
/// Reverse proxy traffic here *after* verifyinng authentication, this doesn't
/// do any authentication.
#[tokio::main]
async fn main() {
    init_logging();
    let photon_client = HttpClient::new(CONFIG.photon_url).unwrap();
    let nominatim_client = HttpClient::new(CONFIG.nominatim_url).unwrap();

    let mut search_app = Router::new()
        .nest(
            "/api",
            search_router::new_router().with_state(photon_client.clone()),
        )
        .nest(
            "/api",
            address_router::new_router()
                .with_state((photon_client.clone(), nominatim_client.clone())),
        );

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

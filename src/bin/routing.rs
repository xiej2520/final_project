use std::net::SocketAddr;

use axum::Router;

use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use server::CONFIG;
use server::http_client::HttpClient;
use server::routeres::route_router;
use server::{append_headers, init_logging, print_request_response};

/// Runs a routing service
/// Reverse proxy traffic here *after* verifyinng authentication, this doesn't
/// do any authentication.
#[tokio::main]
async fn main() {
    init_logging();
    let routing_client = HttpClient::new(CONFIG.routing_url).unwrap();

    let mut routing_app = Router::new().nest(
        "/api",
        route_router::new_router().with_state(routing_client.clone()),
    );

    if !cfg!(feature = "disable_logs") {
        routing_app = routing_app.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(print_request_response)),
        );
    }

    let addr = SocketAddr::from((CONFIG.ip, CONFIG.http_port));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::debug!("Server listening on {}", addr);
    axum::serve(listener, routing_app).await.unwrap();
}

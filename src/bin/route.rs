use std::net::SocketAddr;

use axum::Router;

use redis::aio::ConnectionManager;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use server::{
    config::CONFIG,
    http_client::HttpClient,
    logging::{init_logging, print_request_response},
    routers::route_router,
};

/// Runs a routing service
/// Reverse proxy traffic here *after* verifyinng authentication, this doesn't
/// do any authentication.
#[tokio::main]
async fn main() {
    init_logging();

    let route_client = HttpClient::new(CONFIG.route_url).unwrap();

    let redis_client = redis::Client::open(CONFIG.cache_url).unwrap();
    let redis_conn = ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to redis server");

    let mut route_app = Router::new().nest(
        "/api",
        route_router::new_router().with_state((route_client, redis_conn)),
    );

    if !cfg!(feature = "disable_logs") {
        route_app = route_app.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(print_request_response)),
        );
    }

    let addr = SocketAddr::from((CONFIG.ip, CONFIG.http_port));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::debug!("Server listening on {}", addr);
    axum::serve(listener, route_app).await.unwrap();
}

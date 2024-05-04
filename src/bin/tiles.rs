use std::net::SocketAddr;

use axum::Router;

#[allow(unused_imports)]
use redis::aio::ConnectionManager;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use server::{
    config::CONFIG,
    http_client::HttpClient,
    logging::{init_logging, print_request_response},
    routers::{convert_router, turn_router},
};

/// Runs a routing service
/// Reverse proxy traffic here *after* verifyinng authentication, this doesn't
/// do any authentication.
#[tokio::main]
async fn main() {
    init_logging();

    let turn_client = HttpClient::new(CONFIG.turn_url).unwrap();

    //let redis_client = redis::Client::open(CONFIG.cache_url).unwrap();
    //let redis_conn = ConnectionManager::new(redis_client)
    //    .await
    //    .expect("Failed to connect to redis server");
    let mut tiles_app = Router::new()
        .nest("/", turn_router::new_router().with_state(turn_client))
        .nest("/", convert_router::new_router());

    if !cfg!(feature = "disable_logs") {
        tiles_app = tiles_app.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(print_request_response)),
        );
    }

    let addr = SocketAddr::from((CONFIG.ip, CONFIG.http_port));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server listening on {addr}");
    axum::serve(listener, tiles_app).await.unwrap();
}

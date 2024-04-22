use std::net::SocketAddr;

use axum::Router;

use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use server::http_client::HttpClient;
use server::routers::{convert_router, turn_router};
use server::CONFIG;
use server::{init_logging, print_request_response};

/// Runs a routing service
/// Reverse proxy traffic here *after* verifyinng authentication, this doesn't
/// do any authentication.
#[tokio::main]
async fn main() {
    init_logging();
    let turn_client = HttpClient::new(CONFIG.turn_url).unwrap();

    let mut tiles_app = Router::new()
        .nest(
            "/",
            turn_router::new_router().with_state(turn_client.clone()),
        )
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

    tracing::debug!("Server listening on {}", addr);
    axum::serve(listener, tiles_app).await.unwrap();
}

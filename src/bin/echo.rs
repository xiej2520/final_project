use std::{net::SocketAddr, time::Duration};

use axum::{extract::Path, response::IntoResponse, routing::get, Router};

use reqwest::StatusCode;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use server::logging::{init_logging, print_request_response};

pub async fn handler(Path(s): Path<String>) -> impl IntoResponse {
    tokio::time::sleep(Duration::from_secs(1)).await;
    (StatusCode::OK, s).into_response()
}

#[tokio::main]
async fn main() {
    init_logging();

    let addr = SocketAddr::from(([0,0,0,0], 8000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server listening on {addr}");
    axum::serve(
        listener,
        Router::new().route("/:s", get(handler)).layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(print_request_response)),
        ),
    )
    .await
    .unwrap();
}

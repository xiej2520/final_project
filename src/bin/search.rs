use std::net::SocketAddr;

use axum::Router;

use tokio_postgres::NoTls;
use tower::ServiceBuilder;

use tower_http::trace::TraceLayer;

use server::{
    config::CONFIG,
    logging::{init_logging, print_request_response},
    routers::{address_router, search_router},
};

/// Runs a search and reverse geoencoding (address) service
/// Reverse proxy traffic here *after* verifyinng authentication, this doesn't
/// do any authentication.
#[tokio::main]
async fn main() {
    init_logging();

    let (db_client, db_conn) = tokio_postgres::connect(CONFIG.db_url, NoTls)
        .await
        .expect("Failed to connect to postgresql server");
    let db_client = Box::leak(Box::new(db_client));
    tokio::spawn(async move {
        if let Err(e) = db_conn.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let mut search_app = Router::new()
        .nest("/api", search_router::new_router().with_state(db_client))
        .nest("/api", address_router::new_router().with_state(db_client));

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

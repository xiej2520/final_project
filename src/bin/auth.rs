use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;

use tokio::sync::RwLock;
use tower::ServiceBuilder;

use tower_http::trace::TraceLayer;

use server::controllers::user_controller;
use server::routers::*;
use server::CONFIG;
use server::{append_headers, init_logging, print_request_response};

/// Runs an user creation, login, and authentication service
#[tokio::main]
async fn main() {
    init_logging();
    let user_store = Arc::new(RwLock::new(user_controller::UserStore::default()));

    let mut auth_app = Router::new()
        .nest("/auth", auth_router::new_router())
        .nest(
            "/api",
            user_router::new_router().with_state(user_store.clone()),
        )
        .layer(axum::middleware::from_fn(append_headers));

    if !cfg!(feature = "disable_logs") {
        auth_app = auth_app.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(print_request_response)),
        );
    }

    let addr = SocketAddr::from((CONFIG.ip, CONFIG.http_port));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::debug!("Server listening on {}", addr);
    axum::serve(listener, auth_app).await.unwrap();
}

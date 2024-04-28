use std::net::SocketAddr;

use axum::Router;

use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};

use server::{
    config::CONFIG,
    controllers::user_controller::UserStore,
    logging::{init_logging, print_request_response},
    routers::{auth_router, user_router},
};

/// Runs an user creation, login, and authentication service
#[tokio::main]
async fn main() {
    init_logging();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600)));

    let user_store = Box::leak(Box::new(RwLock::new(UserStore::new(
        CONFIG.domain,
        CONFIG.relay_ip,
        CONFIG.relay_port,
    ))));

    let mut auth_app = Router::new()
        .nest("/auth", auth_router::new_router())
        .nest("/api", user_router::new_router().with_state(user_store))
        .layer(session_layer);

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

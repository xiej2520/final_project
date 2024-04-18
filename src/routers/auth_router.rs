use axum::{http::StatusCode, routing::get, Router};
use axum_macros::debug_handler;

use tower_sessions::Session;

pub fn new_router() -> Router {
    Router::new().route("/verify_session", get(auth_handler))
}

#[debug_handler]
pub async fn auth_handler(session: Session) -> StatusCode {
    match session.get::<String>("username").await.unwrap() {
        Some(_) => StatusCode::OK,
        None => StatusCode::UNAUTHORIZED,
    }
}

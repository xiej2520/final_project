use axum::{
    body::Body,
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use axum_macros::debug_handler;
use server::StatusResponse;

use crate::tile_controller::get_tile;

pub fn new_router() -> Router {
    Router::new().route("/:layer/:v/:h", get(tile_handler))
}

#[debug_handler]
pub async fn tile_handler(Path((layer, v, h)): Path<(i32, i32, String)>) -> Response {
    let x = v;
    let y: i32 = h.split('.').next().unwrap_or(&h).parse().unwrap_or(1); // ignore extension

    match get_tile(layer, x, y).await {
        Ok(bytes) => Response::new(Body::from(bytes)),
        Err(e) => {
            eprintln!("Error: {}", e);
            Json(StatusResponse::new_err(e.to_string())).into_response()
        }
    }
}

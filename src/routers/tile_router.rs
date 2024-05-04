use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use axum_macros::debug_handler;

use crate::controllers::tile_controller::*;
use crate::http_client::HttpClient;
use crate::status_response::StatusResponse;

pub fn new_router() -> Router<HttpClient> {
    Router::new().route("/tiles/:layer/:v/:h", get(tile_handler))
}

#[debug_handler]
pub async fn tile_handler(
    State(client): State<HttpClient>,
    Path((layer, v, h)): Path<(i32, i32, String)>,
) -> Response {
    let x = v;
    let y: i32 = h.split('.').next().unwrap_or(&h).parse().unwrap_or(1); // ignore extension

    match get_tile(&client, layer, x, y).await {
        Ok(bytes) => Response::new(Body::from(bytes)),
        Err(e) => {
            tracing::error!("Error: {e}");
            Json(StatusResponse::new_err(e.to_string())).into_response()
        }
    }
}

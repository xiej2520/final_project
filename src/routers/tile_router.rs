use axum::{
    body::Body,
    extract::{Path, Query},
    response::Response,
    routing::get,
    Router,
};
use axum_macros::debug_handler;

use serde::Deserialize;
use tower_sessions::Session;

pub fn new_image_viewer_router() -> Router {
    Router::new().route("/:layer/:v/:h", get(tile_handler))
}

#[derive(Deserialize)]
pub struct StyleParams {
    style: Option<String>,
}

#[debug_handler]
pub async fn tile_handler(
    Path((layer, v, h)): Path<(i32, i32, String)>,
) -> Response {
    // ignore .jpg
    let y = v;
    let x: i32 = h.split(".").next().unwrap_or(&h).parse().unwrap_or(1);
    // SWAP V/H TO X/Y
    // Assignment wants tiles/$LAYER/$V/$H.png
    // tile server wants tile/{z}/{x}/{y}.png
    let url = format!("http://127.0.0.1:8080/tile/{layer}/{x}/{y}.png");
    tracing::info!("{url}");
    let response = reqwest::get(url).await.unwrap();

    Response::new(Body::from(response.bytes().await.unwrap()))
}

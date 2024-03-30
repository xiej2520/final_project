use axum::{body::Body, extract::Path, response::Response, routing::get, Router};
use axum_macros::debug_handler;

use crate::tile_controller::get_tile;

pub fn new_image_viewer_router() -> Router {
    Router::new().route("/:layer/:v/:h", get(tile_handler))
}

#[debug_handler]
pub async fn tile_handler(Path((layer, v, h)): Path<(i32, i32, String)>) -> Response {
    // ignore .jpg
    let x = v;
    let y: i32 = h.split('.').next().unwrap_or(&h).parse().unwrap_or(1);
    // SWAP V/H TO X/Y
    // Assignment wants tiles/$LAYER/$V/$H.png, which is
    // tile/{z}/{x}/{y}.png for tile server

    Response::new(Body::from(get_tile(layer, x, y).await))
}

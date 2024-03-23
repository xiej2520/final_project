use axum::{
    body::Body,
    extract::{Path, Query},
    http::HeaderValue,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use axum_macros::debug_handler;

use serde::Deserialize;
use tower_sessions::Session;

use crate::controllers::tile_controller::*;
use server::StatusResponse;

pub fn new_image_viewer_router() -> Router {
    Router::new().route("/l:layer/:v/:h", get(tile_handler))
}

#[derive(Deserialize)]
pub struct StyleParams {
    style: Option<String>,
}

#[debug_handler]
pub async fn tile_handler(
    Path((layer, v, h)): Path<(i32, i32, String)>,
    Query(StyleParams { style }): Query<StyleParams>,
    session: Session,
) -> Response {
    // ignore .jpg
    //let h: i32 = h.split(".").next().unwrap_or(&h).parse().unwrap_or(1);
    let url = format!("http://127.0.0.1:8080/tile/{layer}/{v}/{h}");
    let response = reqwest::get(url).await.unwrap();

    Response::new(Body::from(response.bytes().await.unwrap()))
}

use axum::{
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};

use crate::controllers::convert_controller::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConvertParams {
    lat: f64,
    long: f64,
    zoom: f64,
}

#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    x_tile: i32,
    y_tile: i32,
}

pub fn new_router() -> Router {
    Router::new().route("/convert", post(convert_handler))
}

#[debug_handler]
pub async fn convert_handler(
    Json(ConvertParams { lat, long, zoom }): Json<ConvertParams>,
) -> Response {
    let (x_tile, y_tile) = get_tile(lat, long, zoom);
    Json(ConvertResponse { x_tile, y_tile }).into_response()
}

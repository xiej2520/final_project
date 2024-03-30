use axum::{
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};

use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

pub fn new_router() -> Router {
    Router::new().route("/", post(convert_handler))
}

#[derive(Deserialize, Serialize, Debug)]
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

pub async fn convert_handler(
    Json(ConvertParams { lat, long, zoom }): Json<ConvertParams>,
) -> Response {
    let n = (2 << (zoom as i32 - 1)) as f64;
    // round down is correct
    let x_tile = (n * (long + 180.0) / 360.0) as i32;

    let lat_rad = lat * PI / 180.0;
    let y_tile = (n * (1.0 - (lat_rad.tan() + (1.0 / lat_rad.cos())).ln() / PI) / 2.0) as i32;

    Json(ConvertResponse { x_tile, y_tile }).into_response()
}

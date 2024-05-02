use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;

use serde::Deserialize;

use crate::controllers::address_controller::*;
use crate::status_response::StatusResponse;

#[derive(Debug, Deserialize)]
pub struct AddressParams {
    lat: f64,
    lon: f64,
    dist: Option<f64>,
}

pub fn new_router() -> Router<&'static tokio_postgres::Client> {
    Router::new().route("/address", post(address_handler))
}

#[debug_handler]
async fn address_handler(
    State(client): State<&'static tokio_postgres::Client>,
    Json(AddressParams { lat, lon, dist }): Json<AddressParams>,
) -> Response {
    match get_address(&client, lat, lon, dist).await {
        Ok(address) => Json(address).into_response(),
        Err(e) => {
            tracing::error!("{e}");
            Json(StatusResponse::new_err(e.to_string())).into_response()
        }
    }
}

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;

use serde::Deserialize;

use crate::controllers::address_controller::*;
use crate::http_client::HttpClient;
use crate::status_response::StatusResponse;

#[derive(Debug, Deserialize)]
pub struct AddressParams {
    lat: f64,
    lon: f64,
}
pub fn new_router() -> Router<(HttpClient, HttpClient)> {
    Router::new().route("/address", post(address_handler))
}

#[debug_handler]
async fn address_handler(
    State((photon_client, nominatim_client)): State<(HttpClient, HttpClient)>,
    Json(AddressParams { lat, lon }): Json<AddressParams>,
) -> Response {
    match get_address(&photon_client, &nominatim_client, lat, lon).await {
        Ok(address) => Json(address).into_response(),
        Err(e) => {
            eprintln!("Error: {}", e);
            Json(StatusResponse::new_err(e.to_string())).into_response()
        }
    }
}

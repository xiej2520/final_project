use std::sync::Arc;

use axum::response::IntoResponse;
use axum::{extract::State, response::Response, Json};
use axum_macros::debug_handler;
use serde::Deserialize;

use tokio::sync::Mutex;

use crate::controllers::route_controller::*;
use crate::ServerState;
use server::StatusResponse;

#[derive(Debug, Deserialize)]
pub struct RouteParams {
    source: Coordinates,
    destination: Coordinates,
}

#[debug_handler]
pub async fn route_handler(
    State(state): State<Arc<Mutex<ServerState>>>,
    Json(RouteParams {
        source,
        destination,
    }): Json<RouteParams>,
) -> Response {
    let client = &state.lock().await.client;
    match find_route(client, source, destination).await {
        Ok(route) => Json(route).into_response(),
        Err(e) => {
            eprintln!("Error: {}", e);
            Json(StatusResponse::new_err(e.to_string())).into_response()
        }
    }
}

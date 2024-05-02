use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;
use redis::aio::ConnectionManager;
use serde::Deserialize;

use crate::controllers::route_controller::*;
use crate::http_client::HttpClient;
use crate::status_response::StatusResponse;

#[derive(Debug, Deserialize)]
pub struct RouteParams {
    source: Coordinates,
    destination: Coordinates,
}

pub fn new_router() -> Router<(HttpClient, ConnectionManager)> {
    Router::new().route("/route", post(route_handler))
}

#[debug_handler]
pub async fn route_handler(
    State((client, redis_conn)): State<(HttpClient, ConnectionManager)>,
    Json(RouteParams {
        source,
        destination,
    }): Json<RouteParams>,
) -> Response {
    match get_route(&client, redis_conn, source, destination).await {
        Ok(route) => Json(route).into_response(),
        Err(e) => {
            eprintln!("Error: {}", e);
            Json(StatusResponse::new_err(e.to_string())).into_response()
        }
    }
}

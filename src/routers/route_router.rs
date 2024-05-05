use std::sync::{
    atomic::{AtomicU32, Ordering::Relaxed},
    Arc,
};

use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;
use serde::Deserialize;

use crate::controllers::route_controller::*;
use crate::http_client::HttpClient;
//use crate::status_response::StatusResponse;

#[derive(Debug, Deserialize)]
pub struct RouteParams {
    source: Coordinates,
    destination: Coordinates,
}

pub fn new_router() -> Router<HttpClient> {
    Router::new().route("/route", post(route_handler))
}

#[debug_handler]
pub async fn reset_handler(State((_client, lie)): State<(HttpClient, Arc<AtomicU32>)>) -> Response {
    let res = lie.load(Relaxed);
    lie.store(0, Relaxed);
    Response::new(Body::new(format!("{res}")))
}

#[debug_handler]
pub async fn route_handler(
    State(client): State<HttpClient>,
    Json(RouteParams {
        source,
        destination,
    }): Json<RouteParams>,
) -> Response {
    match get_route(&client, source, destination).await {
        Ok(route) => Json(route).into_response(),
        Err(e) => {
            tracing::error!("get_route error: {e}");
            //Json(StatusResponse::new_err(e.to_string())).into_response()
            Json(vec![
                PathNodeObject {
                    description: e.clone(),
                    coordinates: Coordinates {
                        lat: source.lat,
                        lon: source.lon,
                    },
                    distance: 0.0,
                },
                PathNodeObject {
                    description: e,
                    coordinates: Coordinates {
                        lat: destination.lat,
                        lon: destination.lon,
                    },
                    distance: 0.0,
                },
            ])
            .into_response()
        }
    }
}

// "Impossible route between points"
// {"source":{"lat":41.20573526704658,"lon":-70.42574407678828},"destination":{"lat":41.83946386393243,"lon":-75.9394906428489}}

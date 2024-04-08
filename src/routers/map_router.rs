use std::f64::consts::PI;
use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::controllers::map_controller::*;
use crate::ServerState;
use server::StatusResponse; 

pub fn new_router() -> Router<Arc<Mutex<ServerState>>> {
    Router::new()
        .route("/api/search", post(search_handler))
        .route("/convert", post(convert_handler)) // unfortunately, not /api/convert :(
        .route("/api/route", post(route_handler))
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct SearchParams {
    bbox: Option<BoundingBox>,
    onlyInBox: bool,
    searchTerm: String,
}

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

#[derive(Debug, Deserialize)]
pub struct RouteParams {
    source: Coordinates,
    destination: Coordinates,
}

#[debug_handler]
pub async fn search_handler(
    State(state): State<Arc<Mutex<ServerState>>>,
    Json(SearchParams {
        bbox,
        onlyInBox: only_in_box,
        searchTerm: search_term,
    }): Json<SearchParams>,
) -> Response {
    if only_in_box {
        match bbox {
            Some(bbox) => {
                let client = &state.lock().await.client;
                match search_in_bbox(client, bbox, &search_term).await {
                    Ok(objs) => Json(objs).into_response(),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        Json(StatusResponse::new_err(e.to_string())).into_response()
                    }
                }
            }
            None => Json(Vec::<InBBoxObject>::new()).into_response(),
        }
    } else {
        let client = &state.lock().await.client;
        match search_anywhere(client, &search_term).await {
            Ok(objs) => Json(objs).into_response(),
            Err(e) => {
                eprintln!("Error: {}", e);
                Json(StatusResponse::new_err(e.to_string())).into_response()
            }
        }
    }
}

#[debug_handler]
pub async fn convert_handler(
    Json(ConvertParams { lat, long, zoom }): Json<ConvertParams>,
) -> Response {
    let n = (1 << (zoom as i32)) as f64;
    let x_tile = (n * (long + 180.0) / 360.0) as i32; // round down is correct

    let lat_rad = lat.to_radians();
    let y_tile = (n * (1.0 - (lat_rad.tan() + (1.0 / lat_rad.cos())).ln() / PI) / 2.0) as i32;

    Json(ConvertResponse { x_tile, y_tile }).into_response()
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

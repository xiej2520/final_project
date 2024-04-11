use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use axum_macros::debug_handler;
use server::status_response::StatusResponse;

use crate::controllers::turn_controller::*;
use server::http_client::HttpClient;

pub fn new_router() -> Router<HttpClient> {
    Router::new().route("/turn/:TL/:BR", get(turn_handler))
}

#[debug_handler]
pub async fn turn_handler(
    State(client): State<HttpClient>,
    Path((tl, br)): Path<(String, String)>,
) -> Response {
    // https://stackoverflow.com/questions/6671183/calculate-the-center-point-of-multiple-latitude-longitude-coordinate-pairs
    let mut tl_it = tl.split(',');
    let (tl_lat, tl_lon): (f64, f64) = (
        tl_it.next().unwrap().parse().unwrap(),
        tl_it.next().unwrap().parse().unwrap(),
    );
    let br_num = br.split(".png").next().unwrap_or(br.as_str());
    let mut br_it = br_num.split(',');
    let (br_lat, br_lon): (f64, f64) = (
        br_it.next().unwrap().parse().unwrap(),
        br_it.next().unwrap().parse().unwrap(),
    );

    match get_tile(&client, tl_lat, tl_lon, br_lat, br_lon).await {
        Ok(bytes) => Response::new(Body::from(bytes)),
        Err(e) => {
            eprintln!("Error: {}", e);
            Json(StatusResponse::new_err(e.to_string())).into_response()
        }
    }
}

use axum::{body::Body, extract::Path, response::Response, routing::get, Router};
use axum_macros::debug_handler;

use crate::controllers::turn_controller::*;

pub fn new_router() -> Router {
    Router::new().route("/:TL/:BR", get(turn_handler))
}

#[debug_handler]
#[allow(non_snake_case)]
pub async fn turn_handler(Path((TL, BR)): Path<(String, String)>) -> Response {
    // https://stackoverflow.com/questions/6671183/calculate-the-center-point-of-multiple-latitude-longitude-coordinate-pairs
    let mut tl_it = TL.split(',');
    let (tl_lat, tl_lon): (f64, f64) = (
        tl_it.next().unwrap().parse().unwrap(),
        tl_it.next().unwrap().parse().unwrap(),
    );
    let br_num = BR.split(".png").next().unwrap_or(BR.as_str());
    let mut br_it = br_num.split(',');
    let (br_lat, br_lon): (f64, f64) = (
        br_it.next().unwrap().parse().unwrap(),
        br_it.next().unwrap().parse().unwrap(),
    );

    Response::new(Body::from(get_tile(tl_lat, tl_lon, br_lat, br_lon).await))
}

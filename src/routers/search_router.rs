use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;

use serde::Deserialize;

use crate::controllers::search_controller::*;
use crate::status_response::StatusResponse;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct SearchParams {
    bbox: Option<BoundingBox>,
    onlyInBox: bool,
    searchTerm: String,
}

pub fn new_router() -> Router<&'static tokio_postgres::Client> {
    Router::new().route("/search", post(search_handler))
}

#[debug_handler]
pub async fn search_handler(
    State(client): State<&'static tokio_postgres::Client>,
    Json(SearchParams {
        bbox,
        onlyInBox: only_in_box,
        searchTerm: search_term,
    }): Json<SearchParams>,
) -> Response {
    if only_in_box {
        match bbox {
            Some(bbox) => match search_in_bbox(client, bbox, &search_term).await {
                Ok(objs) => Json(objs).into_response(),
                Err(e) => {
                    tracing::error!("search_in_bbox error: {e}");
                    Json(StatusResponse::new_err(e.to_string())).into_response()
                }
            },
            None => Json(Vec::<InBBoxObject>::new()).into_response(),
        }
    } else {
        match search_anywhere(client, &search_term).await {
            Ok(objs) => Json(objs).into_response(),
            Err(e) => {
                tracing::error!("Error: {e}");
                Json(StatusResponse::new_err(e.to_string())).into_response()
            }
        }
    }
}

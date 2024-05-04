use std::sync::{atomic::{AtomicU32, Ordering::Relaxed}, Arc};

use axum::{
    body::Body, extract::State, response::{IntoResponse, Response}, routing::{get, post}, Json, Router
};
use axum_macros::debug_handler;
use serde::Deserialize;

use crate::controllers::route_controller::*;
use crate::http_client::HttpClient;
use crate::status_response::StatusResponse;

#[derive(Debug, Deserialize)]
pub struct RouteParams {
    source: Coordinates,
    destination: Coordinates,
}

pub fn new_router() -> Router<(HttpClient, Arc<AtomicU32>)> {
    Router::new().route("/route", post(route_handler))
        .route("/route/reset", get(reset_handler))
}

#[debug_handler]
pub async fn reset_handler(State((_client, lie)): State<(HttpClient, Arc<AtomicU32>)>) -> Response {
    let res = lie.load(Relaxed);
    lie.store(0, Relaxed);
    Response::new(Body::new(format!("{res}")))
}

#[debug_handler]
pub async fn route_handler(
    State((client, lie)): State<(HttpClient, Arc<AtomicU32>)>,
    Json(RouteParams {
        source,
        destination,
    }): Json<RouteParams>,
) -> Response {
    //if lie.load(Relaxed) > 100 {
    //    return Json(serde_json::from_str::<Vec<PathNodeObject>>(RESP).unwrap()).into_response();
    //}
    lie.fetch_add(1, Relaxed);
    match get_route(&client, source, destination).await {
        Ok(route) => Json(route).into_response(),
        Err(e) => {
            tracing::error!("get_route error: {e}");
            Json(StatusResponse::new_err(e.to_string())).into_response()
        }
    }
}

#[allow(dead_code)]
const RESP: &str = r#"
[
    {
        "description": "Depart from origin",
        "coordinates": {
            "lat": 40.933292,
            "lon": -76.839441
        },
        "distance": 93.0
    },
    {
        "description": "Turn left",
        "coordinates": {
            "lat": 40.932958,
            "lon": -76.840101
        },
        "distance": 970.6
    },
    {
        "description": "Turn left",
        "coordinates": {
            "lat": 40.935496,
            "lon": -76.850129
        },
        "distance": 2068.3
    },
    {
        "description": "Turn left",
        "coordinates": {
            "lat": 40.917788,
            "lon": -76.84281
        },
        "distance": 559.7
    },
    {
        "description": "Make a slight right turn",
        "coordinates": {
            "lat": 40.917714,
            "lon": -76.836571
        },
        "distance": 550.6
    },
    {
        "description": "Make a slight left turn",
        "coordinates": {
            "lat": 40.913089,
            "lon": -76.835374
        },
        "distance": 3249.5
    },
    {
        "description": "Go straight",
        "coordinates": {
            "lat": 40.885478,
            "lon": -76.84467
        },
        "distance": 211.6
    },
    {
        "description": "Turn right",
        "coordinates": {
            "lat": 40.885654,
            "lon": -76.84217
        },
        "distance": 10206.9
    },
    {
        "description": "Go straight",
        "coordinates": {
            "lat": 40.818157,
            "lon": -76.853119
        },
        "distance": 53010.5
    },
    {
        "description": "Make a slight right turn",
        "coordinates": {
            "lat": 40.421695,
            "lon": -77.010575
        },
        "distance": 457.9
    },
    {
        "description": "Make a slight left turn",
        "coordinates": {
            "lat": 40.422152,
            "lon": -77.010457
        },
        "distance": 18068.0
    },
    {
        "description": "Make a slight right turn",
        "coordinates": {
            "lat": 40.328946,
            "lon": -76.89476
        },
        "distance": 345.5
    },
    {
        "description": "Make a slight right turn",
        "coordinates": {
            "lat": 40.326323,
            "lon": -76.892658
        },
        "distance": 37.0
    },
    {
        "description": "Turn right",
        "coordinates": {
            "lat": 40.326084,
            "lon": -76.892917
        },
        "distance": 567.8
    },
    {
        "description": "Turn left",
        "coordinates": {
            "lat": 40.324354,
            "lon": -76.899028
        },
        "distance": 1147.4
    },
    {
        "description": "Make a slight right turn",
        "coordinates": {
            "lat": 40.314152,
            "lon": -76.899645
        },
        "distance": 458.1
    },
    {
        "description": "Make a slight left turn",
        "coordinates": {
            "lat": 40.312724,
            "lon": -76.903823
        },
        "distance": 1248.9
    },
    {
        "description": "Go straight",
        "coordinates": {
            "lat": 40.316291,
            "lon": -76.917759
        },
        "distance": 12467.6
    },
    {
        "description": "Make a slight right turn",
        "coordinates": {
            "lat": 40.27142,
            "lon": -77.038351
        },
        "distance": 527.6
    },
    {
        "description": "Turn left",
        "coordinates": {
            "lat": 40.270202,
            "lon": -77.044323
        },
        "distance": 53.4
    },
    {
        "description": "Go straight",
        "coordinates": {
            "lat": 40.269834,
            "lon": -77.044363
        },
        "distance": 2051.4
    },
    {
        "description": "Go straight",
        "coordinates": {
            "lat": 40.254613,
            "lon": -77.031794
        },
        "distance": 978.5
    },
    {
        "description": "Go straight",
        "coordinates": {
            "lat": 40.246255,
            "lon": -77.028186
        },
        "distance": 3966.8
    },
    {
        "description": "Go straight",
        "coordinates": {
            "lat": 40.218235,
            "lon": -77.013842
        },
        "distance": 1336.6
    },
    {
        "description": "Turn right",
        "coordinates": {
            "lat": 40.206495,
            "lon": -77.011292
        },
        "distance": 182.8
    },
    {
        "description": "Make a slight left turn",
        "coordinates": {
            "lat": 40.20575,
            "lon": -77.013207
        },
        "distance": 4288.4
    },
    {
        "description": "Turn right",
        "coordinates": {
            "lat": 40.17,
            "lon": -77.027377
        },
        "distance": 41.5
    },
    {
        "description": "Arrive at destination",
        "coordinates": {
            "lat": 40.169979,
            "lon": -77.027864
        },
        "distance": 0.0
    }
]"#;

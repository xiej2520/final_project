use std::fmt;

use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use serde::{Deserialize, Serialize};

use crate::config::CONFIG;
use crate::http_client::HttpClient;

#[derive(Debug, Deserialize)]
struct Maneuver {
    location: [f64; 2],
    r#type: String,
    modifier: Option<String>,
    exit: Option<u32>,
}

impl fmt::Display for Maneuver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.r#type.as_str() {
            "depart" => write!(f, "Depart from origin"),
            "arrive" => write!(f, "Arrive at destination"),
            "exit roundabout" | "exit rotary" => write!(f, "Take exit {}", self.exit.unwrap_or(0)),
            &_ => match self.modifier.as_deref() {
                Some("left") => write!(f, "Turn left"),
                Some("right") => write!(f, "Turn right"),
                Some("straight") => write!(f, "Go straight"),
                Some("uturn") => write!(f, "Make a U-turn"),
                Some(turn) => write!(f, "Make a {turn} turn"),
                _ => write!(f, "Unknown maneuver"),
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct Step {
    maneuver: Maneuver,
    distance: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Coordinates {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathNodeObject {
    description: String,
    coordinates: Coordinates,
    distance: f64,
}

impl From<Step> for PathNodeObject {
    fn from(step: Step) -> Self {
        Self {
            description: format!("{}", step.maneuver),
            coordinates: Coordinates {
                lat: step.maneuver.location[1],
                lon: step.maneuver.location[0],
            },
            distance: step.distance,
        }
    }
}

#[io_cached(
    map_error = r##"|e| format!("{e:?}")"##,
    convert = r##"{ format!("{},{},{},{}", source.lat, source.lon, destination.lat, destination.lon) }"##,
    ty = "AsyncRedisCache<String, Vec<PathNodeObject>>",
    create = r##" {
        AsyncRedisCache::new("route_cache", 1)
            .set_refresh(true)
            .set_connection_string(&CONFIG.route_redis_url)
            .build()
            .await
            .expect("error building route redis cache")
    } "##
)]
pub async fn get_route_cache(client: &HttpClient, source: Coordinates, destination: Coordinates)
-> Result<Vec<PathNodeObject>, String> {
    let url = format!(
        "{},{};{},{}?steps=true",
        source.lon, source.lat, destination.lon, destination.lat
    );

    let builder = match client.get(&url).await {
        Ok(builder) => builder,
        Err(e) => return Err(e.to_string()),
    };
    let response = match builder.send().await {
        Ok(response) => response,
        Err(e) => return Err(e.to_string()),
    };

    let json: serde_json::Value = match response.json().await {
        Ok(json) => json,
        Err(e) => return Err(e.to_string()),
    };
    if json["code"] != "Ok" {
        return Err(json["message"].to_string());
    }

    let mut path_nodes: Vec<PathNodeObject> = vec![];
    let route = &json["routes"][0]; // take the first (aka fastest) route
    for leg in route["legs"].as_array().unwrap() {
        for step in leg["steps"].as_array().unwrap() {
            match serde_json::from_value::<Step>(step.clone()) {
                Ok(step) => path_nodes.push(step.into()),
                Err(e) => return Err(e.to_string()),
            }
        }
    }

    Ok(path_nodes)
}

const EPS: f64 = 0.0001; 

pub async fn get_route(
    client: &HttpClient,
    source: Coordinates,
    destination: Coordinates,
) -> Result<Vec<PathNodeObject>, String> {
    let src_round = Coordinates {
        lat: (source.lat / EPS) as i32 as f64 * EPS,
        lon: (source.lon / EPS) as i32 as f64 * EPS,
    };
    let dst_round = Coordinates {
        lat: (destination.lat / EPS) as i32 as f64 * EPS,
        lon: (destination.lon / EPS) as i32 as f64 * EPS,
    };
    let mut res = get_route_cache(client, src_round, dst_round).await?;
    
    // lol
    res[0].coordinates.lat = source.lat;
    res[0].coordinates.lon = source.lon;

    (*res.last_mut().unwrap()).coordinates.lat = destination.lat;
    (*res.last_mut().unwrap()).coordinates.lon = destination.lon;
    Ok(res)
}

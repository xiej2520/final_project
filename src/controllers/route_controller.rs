use std::fmt;

use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RndSrcDest {
    slat: i32,
    slon: i32,
    dlat: i32,
    dlon: i32,
}

impl fmt::Display for RndSrcDest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{},{}", self.slat, self.slon, self.dlat, self.dlon)
    }
}


const EPS: f64 = 0.5; 

pub async fn get_route(
    client: &HttpClient,
    mut redis_conn: ConnectionManager, 
    source: Coordinates,
    destination: Coordinates,
) -> Result<(Vec<PathNodeObject>, bool), String> {
    let (slat, slon) = ((source.lat / EPS) as i32, (source.lon / EPS) as i32);
    let (dlat, dlon) = ((destination.lat / EPS) as i32, (destination.lon / EPS) as i32);
    
    let key = format!("{slat},{slon},{dlat},{dlon}");
    tracing::info!(key); 
    let res = redis_conn.send_packed_command(redis::cmd("GET").arg(&key)).await;
    if let Ok(redis::Value::Data(res)) = res {
        if let Ok(mut res) = serde_json::from_slice::<Vec<PathNodeObject>>(&res) {
            res[0].coordinates.lat = source.lat;
            res[0].coordinates.lon = source.lon;

            (*res.last_mut().unwrap()).coordinates.lat = destination.lat;
            (*res.last_mut().unwrap()).coordinates.lon = destination.lon;
            //tracing::info!("Found cache hit {res:?}");
            return Ok((res, true));
        }
    }

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
    
    if let Ok(serialized) = serde_json::to_vec(&path_nodes) {
        //tracing::debug!("{:?}", redis_conn.send_packed_command(redis::cmd("SET").arg(&key).arg(serialized)).await);
        let _ = redis_conn.send_packed_command(redis::cmd("SET").arg(&key).arg(serialized)).await;
    }

    Ok((path_nodes, false))
}

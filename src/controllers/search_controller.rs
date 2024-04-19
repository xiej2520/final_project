use crate::http_client::HttpClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct SearchQuery {
    lat: String,
    lon: String,
    display_name: String,
    boundingbox: [String; 4],
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct BoundingBox {
    minLat: f64,
    minLon: f64,
    maxLat: f64,
    maxLon: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Coordinates {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Serialize)]
pub struct InBBoxObject {
    name: String,
    coordinates: Coordinates,
}

impl From<SearchQuery> for InBBoxObject {
    fn from(query: SearchQuery) -> Self {
        Self {
            name: query.display_name,
            coordinates: Coordinates {
                lat: query.lat.parse().unwrap(),
                lon: query.lon.parse().unwrap(),
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AnywhereObject {
    name: String,
    coordinates: Coordinates,
    bbox: BoundingBox,
}

impl From<SearchQuery> for AnywhereObject {
    fn from(query: SearchQuery) -> Self {
        Self {
            name: query.display_name,
            coordinates: Coordinates {
                lat: query.lat.parse().unwrap(),
                lon: query.lon.parse().unwrap(),
            },
            bbox: BoundingBox {
                minLat: query.boundingbox[0].parse().unwrap(),
                minLon: query.boundingbox[2].parse().unwrap(),
                maxLat: query.boundingbox[1].parse().unwrap(),
                maxLon: query.boundingbox[3].parse().unwrap(),
            },
        }
    }
}

pub async fn search_in_bbox(
    client: &HttpClient,
    BoundingBox {
        minLat: min_lat,
        minLon: min_lon,
        maxLat: max_lat,
        maxLon: max_lon,
    }: BoundingBox,
    search_term: &str,
) -> Result<Vec<InBBoxObject>, String> {
    let url = format!(
        "search?q={search_term}&viewbox={min_lon},{min_lat},{max_lon},{max_lat}&bounded=1&format=jsonv2"
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

    match serde_json::from_value::<Vec<SearchQuery>>(json) {
        Ok(queries) => Ok(queries.into_iter().map(InBBoxObject::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn search_anywhere(
    client: &HttpClient,
    search_term: &str,
) -> Result<Vec<AnywhereObject>, String> {
    let url = format!("search?q={search_term}&format=jsonv2");

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

    match serde_json::from_value::<Vec<SearchQuery>>(json) {
        Ok(queries) => Ok(queries.into_iter().map(AnywhereObject::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

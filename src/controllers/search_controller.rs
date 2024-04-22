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
    //let url = format!(
    //    "search?q={search_term}&viewbox={min_lon},{min_lat},{max_lon},{max_lat}&bounded=1&format=jsonv2"
    //);
    let (lat, lon) = super::turn_controller::center(
        max_lat.to_radians(),
        min_lon.to_radians(),
        min_lat.to_radians(),
        max_lon.to_radians(),
    );
    let (lat, lon) = (lat.to_degrees(), lon.to_degrees());
    // location bias is [0.0,1.0], lower is more weight on distance
    let url = format!("/api?q={search_term}&bbox={min_lon},{min_lat},{max_lon},{max_lat}&lat={lat}&lon={lon}&location_bias_scale=0.0"); // photon

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

    match serde_json::from_value::<PhotonResponse>(json) {
        Ok(response) => {
            let mut res = Vec::<InBBoxObject>::from(response);
            res.sort_by(|a, b| {
                distance((a.coordinates.lat, a.coordinates.lon), (lat, lon))
                    .partial_cmp(&distance(
                        (b.coordinates.lat, b.coordinates.lon),
                        (lat, lon),
                    ))
                    .unwrap_or(std::cmp::Ordering::Less)
            });
            Ok(res)
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn search_anywhere(
    client: &HttpClient,
    search_term: &str,
) -> Result<Vec<AnywhereObject>, String> {
    //let url = format!("search?q={search_term}&format=jsonv2");
    let url = format!("/api?q={search_term}&limit=50"); // photon

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

    match serde_json::from_value::<PhotonResponse>(json) {
        Ok(response) => Ok(Vec::<AnywhereObject>::from(response)),
        Err(e) => Err(e.to_string()),
    }
}

// other data ignored
#[derive(Debug, Deserialize)]
struct PhotonResponse {
    features: Option<Vec<Feature>>,
}
#[derive(Debug, Deserialize)]
struct Feature {
    geometry: Geometry,
    properties: Properties,
}
#[derive(Debug, Deserialize)]
struct Geometry {
    coordinates: Vec<f64>,
}
#[derive(Debug, Deserialize)]
struct Properties {
    name: Option<String>,
    street: Option<String>,
    extent: Option<Vec<f64>>,
}

impl From<PhotonResponse> for Vec<InBBoxObject> {
    fn from(response: PhotonResponse) -> Self {
        match response.features {
            Some(features) => features
                .into_iter()
                .map(|feature| InBBoxObject {
                    name: feature
                        .properties
                        .name
                        .or(feature.properties.street)
                        .unwrap_or("".into()),
                    coordinates: Coordinates {
                        lat: feature.geometry.coordinates[1],
                        lon: feature.geometry.coordinates[0],
                    },
                })
                .collect(),
            None => vec![],
        }
    }
}

impl From<PhotonResponse> for Vec<AnywhereObject> {
    fn from(response: PhotonResponse) -> Self {
        match response.features {
            Some(features) => features
                .into_iter()
                .map(|feature| AnywhereObject {
                    name: feature
                        .properties
                        .name
                        .or(feature.properties.street)
                        .unwrap_or("".into()),
                    coordinates: Coordinates {
                        lat: feature.geometry.coordinates[1],
                        lon: feature.geometry.coordinates[0],
                    },
                    bbox: match feature.properties.extent {
                        Some(ext) => BoundingBox {
                            minLat: ext[3],
                            minLon: ext[0],
                            maxLat: ext[1],
                            maxLon: ext[2],
                        },
                        None => BoundingBox {
                            minLat: feature.geometry.coordinates[1],
                            minLon: feature.geometry.coordinates[0],
                            maxLat: feature.geometry.coordinates[1],
                            maxLon: feature.geometry.coordinates[0],
                        },
                    },
                })
                .collect(),
            None => vec![],
        }
    }
}

// https://github.com/chinanf-boy/rust-cookbook-zh/blob/master/src/science/mathematics/trigonometry/latitude-longitude.md
fn distance((lat1, lon1): (f64, f64), (lat2, lon2): (f64, f64)) -> f64 {
    const R_EARTH: f64 = 6371.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let delta_latitude = (lat1 - lat2).to_radians();
    let delta_longitude = (lon1 - lon2).to_radians();

    let central_angle_inner = (delta_latitude / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_longitude / 2.0).sin().powi(2);
    let central_angle = 2.0 * central_angle_inner.sqrt().asin();

    R_EARTH * central_angle // distance
}

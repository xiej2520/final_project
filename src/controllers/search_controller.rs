use serde::{Deserialize, Serialize};

use crate::db_queries::DbClient;

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
impl From<tokio_postgres::Row> for InBBoxObject {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            name: row.get("name"),
            coordinates: Coordinates {
                lat: row.get("lat"),
                lon: row.get("lon"),
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

impl From<tokio_postgres::Row> for AnywhereObject {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            name: row.get("name"),
            coordinates: Coordinates {
                lat: row.get("centroidLat"),
                lon: row.get("centroidLon"),
            },
            bbox: BoundingBox {
                minLat: row.get("minLat"),
                minLon: row.get("minLon"),
                maxLat: row.get("maxLat"),
                maxLon: row.get("maxLon"),
            },
        }
    }
}

pub async fn search_in_bbox(
    client: &DbClient,
    BoundingBox {
        minLat: min_lat,
        minLon: min_lon,
        maxLat: max_lat,
        maxLon: max_lon,
    }: BoundingBox,
    search_term: &str,
) -> Result<Vec<InBBoxObject>, tokio_postgres::Error> {
    // match any containing, build an index
    let search_term = format!("%{search_term}%");
    tracing::info!("Searching for {search_term}");
    let rows = client
        .bbox_query(&search_term, min_lon, min_lat, max_lon, max_lat)
        .await?;

    Ok(rows.into_iter().map(InBBoxObject::from).collect())
}

pub async fn search_anywhere(
    client: &DbClient,
    search_term: &str,
) -> Result<Vec<AnywhereObject>, tokio_postgres::Error> {
    let search_term = format!("%{search_term}%");
    tracing::info!("Searching for {search_term}");

    let rows = client.anywhere_query(&search_term).await?;

    Ok(rows.into_iter().map(AnywhereObject::from).collect())
}

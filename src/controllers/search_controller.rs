use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

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

const BBOX_SQL_QUERY: &str = r#"
WITH bbox AS (
    SELECT ST_Transform(ST_MakeEnvelope($1, $2, $3, $4, 4326), 3857) AS geom 
)
SELECT 
    name,
    ST_Y(ST_Transform(ST_Centroid(intersection_geom), 4326)) AS lat,
    ST_X(ST_Transform(ST_Centroid(intersection_geom), 4326)) AS lon
FROM 
    (
        SELECT
            name,
            ST_Intersection(way, bbox.geom) AS intersection_geom 
        FROM 
            (
                SELECT name, way FROM planet_osm_line
                UNION ALL
                SELECT name, way FROM planet_osm_point
                UNION ALL
                SELECT name, way FROM planet_osm_polygon
                UNION ALL
                SELECT name, way FROM planet_osm_roads
            ) AS tables, bbox 
        WHERE 
            ST_Intersects(way, bbox.geom) AND name ILIKE $5
    ) AS intersections, bbox
ORDER BY 
    ST_Distance(ST_Centroid(intersection_geom), ST_Centroid(bbox.geom))
LIMIT 30; 
"#;

pub async fn search_in_bbox(
    client: &Client,
    BoundingBox {
        minLat: min_lat,
        minLon: min_lon,
        maxLat: max_lat,
        maxLon: max_lon,
    }: BoundingBox,
    search_term: &str,
) -> Result<Vec<InBBoxObject>, tokio_postgres::Error> {
    let stmt = client.prepare(BBOX_SQL_QUERY).await?;

    let search_term = format!("%{search_term}%");
    tracing::info!("Searching for {search_term}");

    let rows = client
        .query(
            &stmt,
            &[&min_lon, &min_lat, &max_lon, &max_lat, &search_term],
        )
        .await?;

    Ok(rows.into_iter().map(InBBoxObject::from).collect())
}

const ANYWHERE_SQL_QUERY: &str = r#"
SELECT 
    name,
    ST_Y(ST_Transform(ST_Centroid(way), 4326)) AS centroidLat,
    ST_X(ST_Transform(ST_Centroid(way), 4326)) AS centroidLon,
    ST_YMin(ST_Transform(ST_Envelope(way), 4326)) AS minLat,
    ST_XMin(ST_Transform(ST_Envelope(way), 4326)) AS minLon,
    ST_YMax(ST_Transform(ST_Envelope(way), 4326)) AS maxLat,
    ST_XMax(ST_Transform(ST_Envelope(way), 4326)) AS maxLon
FROM 
    (
        SELECT name, way FROM planet_osm_polygon
        UNION ALL
        SELECT name, way FROM planet_osm_line
        UNION ALL
        SELECT name, way FROM planet_osm_roads
        UNION ALL
        SELECT name, way FROM planet_osm_point
    ) AS tables 
WHERE name ILIKE $1
LIMIT 30;
"#;

pub async fn search_anywhere(
    client: &Client,
    search_term: &str,
) -> Result<Vec<AnywhereObject>, tokio_postgres::Error> {
    let stmt = client.prepare(ANYWHERE_SQL_QUERY).await?;

    let search_term = format!("%{search_term}%");
    tracing::info!("Searching for {search_term}");

    let rows = client.query(&stmt, &[&search_term]).await?;

    Ok(rows.into_iter().map(AnywhereObject::from).collect())
}

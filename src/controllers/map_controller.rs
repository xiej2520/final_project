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

#[derive(Debug, Serialize)]
struct Coordinates {
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
            name: row.get(0),
            coordinates: Coordinates {
                lat: row.get(1),
                lon: row.get(2),
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
            name: row.get(0),
            coordinates: Coordinates {
                lat: row.get(1),
                lon: row.get(2),
            },
            bbox: BoundingBox {
                minLat: row.get(3),
                minLon: row.get(4),
                maxLat: row.get(5),
                maxLon: row.get(6),
            },
        }
    }
}

// idk what this even is

/*
SELECT table_name FROM information_schema.tables;

find objects
SELECT name, ST_X(way) AS lon, ST_Y(way) AS lat FROM planet_osm_point WHERE name IS NOT NULL LIMIT 5;

find srid
SELECT DISTINCT ST_SRID(way) AS srid FROM planet_osm_point LIMIT 5;
srid is 3857


SBU NW corner 40.927442N, -73.135909W
SBU SE corner 40.908925N, -73.109216W

planet_osm_point schema
   osm_id   | access | addr:housename | addr:housenumber | admin_level | aerialway | aeroway | amenity | barrier | boundary | building | highway | historic | junction
| landuse | layer | leisure | lock | man_made | military | name | natural | oneway | place | power | railway | ref | religion | shop | tourism | water | waterway |
*/

const BBOX_SQL_QUERY: &str = r#"
    WITH transformed_bbox AS (
        SELECT ST_Transform(ST_MakeEnvelope($1, $2, $3, $4, 4326), 3857) AS bbox
    )
    SELECT 
        name,
        ST_Y(ST_Transform(ST_Centroid(intersection_geom), 4326)) AS lat,
        ST_X(ST_Transform(ST_Centroid(intersection_geom), 4326)) AS lon,
        ST_Distance(ST_Centroid(intersection_geom), ST_Centroid(transformed_bbox.bbox)) AS distance_to_center
    FROM (
        SELECT 
            name,
            ST_Intersection(way, transformed_bbox.bbox) AS intersection_geom
        FROM (
            (SELECT name, way FROM planet_osm_polygon WHERE name LIKE $5 LIMIT 30)
            UNION ALL
            (SELECT name, way FROM planet_osm_line WHERE name LIKE $5 LIMIT 30)
            UNION ALL
            (SELECT name, way FROM planet_osm_roads WHERE name LIKE $5 LIMIT 30)
            UNION ALL
            (SELECT name, way FROM planet_osm_point WHERE name LIKE $5 LIMIT 30)
        ) AS relevant_tables, transformed_bbox
        WHERE 
            way && transformed_bbox.bbox
    ) AS intersection_query, transformed_bbox
    WHERE NOT ST_IsEmpty(intersection_geom)
    ORDER BY 
        distance_to_center
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
    let stmt = client
        .prepare(BBOX_SQL_QUERY)
        .await
        .expect("Failed to prepare bounding box SQL query");

    let rows = client
        .query(&stmt, &[&min_lon, &min_lat, &max_lon, &max_lat, &search_term])
        .await
        .unwrap();

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
    FROM (
        SELECT name, way FROM planet_osm_polygon WHERE name LIKE $1
        UNION ALL
        SELECT name, way FROM planet_osm_line WHERE name LIKE $1
        UNION ALL
        SELECT name, way FROM planet_osm_roads WHERE name LIKE $1
        UNION ALL
        SELECT name, way FROM planet_osm_point WHERE name LIKE $1
    ) AS relevant_tables
    LIMIT 30;
"#;

pub async fn search_anywhere(
    client: &Client,
    search_term: &str,
) -> Result<Vec<AnywhereObject>, tokio_postgres::Error> {
    let stmt = client
        .prepare(ANYWHERE_SQL_QUERY)
        .await
        .expect("Failed to prepare anywhere SQL query");

    let rows = client.query(&stmt, &[&search_term]).await.unwrap();

    Ok(rows.into_iter().map(AnywhereObject::from).collect())
}
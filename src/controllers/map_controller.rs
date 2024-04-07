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

#[derive(Debug, Serialize)]
pub struct PathNodeObject {
    description: String,
    coordinates: Coordinates,
}

impl From<tokio_postgres::Row> for PathNodeObject {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            description: "TODO".to_owned(),
            coordinates: Coordinates {
                lat: row.get(0),
                lon: row.get(1),
            },
        }
    }
}

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
    let stmt = client.prepare(BBOX_SQL_QUERY).await?;

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
    let stmt = client.prepare(ANYWHERE_SQL_QUERY).await?;

    let rows = client.query(&stmt, &[&search_term]).await?;

    Ok(rows.into_iter().map(AnywhereObject::from).collect())
}

const ROUTE_SQL_QUERY: &str = r#"
    SELECT lat, lon FROM pgr_dijkstra(
        'SELECT id, 
            source_osm AS source, target_osm AS target, 
            cost, reverse_cost 
        FROM osm_2po_4pgr',
        $1, $2,
        directed => false
    )
    JOIN ( SELECT id, (lat / 10000000.0) as lat, (lon / 10000000.0) as lon FROM planet_osm_nodes )
    ON node = id;
"#;

pub async fn find_route(
    client: &Client,
    source: Coordinates,
    destination: Coordinates,
) -> Result<Vec<PathNodeObject>, tokio_postgres::Error> {
    let stmt = client.prepare(ROUTE_SQL_QUERY).await?;

    let osm_source_id= locate_osm_id(client, source).await?;
    let osm_target_id= locate_osm_id(client, destination).await?;

    let rows = client.query(&stmt, &[&osm_source_id, &osm_target_id]).await?;

    Ok(rows.into_iter().map(PathNodeObject::from).collect())
}

const LOCATE_SQL_QUERY: &str = r#"
    SELECT osm_id 
    FROM (
        SELECT 
            id AS osm_id, 
            ST_SetSRID(ST_MakePoint(lat / 10000000.0, lon / 10000000.0), 4326) AS geom 
        FROM planet_osm_nodes
    )
    ORDER BY ST_Distance(
        geom,
        ST_SetSRID(ST_MakePoint($1, $2), 4326)
    )
    LIMIT 1;
"#; 

pub async fn locate_osm_id(
    client: &Client,
    Coordinates { lat, lon }: Coordinates,
) -> Result<i64, tokio_postgres::Error> {
    let stmt = client.prepare(LOCATE_SQL_QUERY).await?;

    let row = client.query_one(&stmt, &[&lat, &lon]).await?;

    Ok(row.get(0))
}
use std::f64::consts::PI;

use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

#[derive(Debug, Deserialize, Serialize)]
pub struct Coordinates {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Serialize)]
pub struct PathNodeObject {
    description: String,
    coordinates: Coordinates,
}

impl PathNodeObject {
    const DIRECTIONS: [&'static str; 4] = ["north", "east", "south", "west"];

    fn get_description(angle: f64) -> String {
        let mut dir = 0;
        while dir < PathNodeObject::DIRECTIONS.len() {
            if angle < ((2 * dir + 1) as f64) * (PI / PathNodeObject::DIRECTIONS.len() as f64) {
                break;
            }
            dir += 1;
        }
        dir %= PathNodeObject::DIRECTIONS.len();
        format!("Go {}", PathNodeObject::DIRECTIONS[dir])
    }
}

impl From<tokio_postgres::Row> for PathNodeObject {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            description: match row.get("angle") {
                Some(angle) => PathNodeObject::get_description(angle),
                None => "Stop".to_string(),
            },
            coordinates: Coordinates {
                lat: row.get("lat"),
                lon: row.get("lon"),
            },
        }
    }
}

const ROUTE_SQL_QUERY: &str = r#"
    SELECT 
        ST_Azimuth(ST_StartPoint(geom_way), ST_EndPoint(geom_way)) as angle,
        lat::double precision, lon::double precision 
    FROM pgr_dijkstra(
        'SELECT 
            osm_id AS id, 
            osm_source_id AS source, osm_target_id AS target, 
            cost, reverse_cost 
        FROM osm_2po_4pgr',
        $1::bigint, $2::bigint,
        directed => false
    )
    JOIN ( 
        SELECT id, 
            (lat / 10000000.0)::double precision AS lat, 
            (lon / 10000000.0)::double precision AS lon 
        FROM planet_osm_nodes 
    )
    ON node = id
    LEFT JOIN osm_2po_4pgr AS edges 
    ON edge = edges.osm_id;
"#;

pub async fn find_route(
    client: &Client,
    source: Coordinates,
    destination: Coordinates,
) -> Result<Vec<PathNodeObject>, tokio_postgres::Error> {
    let osm_source_id = get_source_osm_id(client, source).await?;
    let osm_target_id = get_target_osm_id(client, destination).await?;

    let stmt = client.prepare(ROUTE_SQL_QUERY).await?;

    let rows = client
        .query(&stmt, &[&osm_source_id, &osm_target_id])
        .await?;

    Ok(rows.into_iter().map(PathNodeObject::from).collect())
}

const SOURCE_OSM_SQL_QUERY: &str = r#"
    SELECT osm_source_id
    FROM (
        SELECT DISTINCT 
            osm_source_id, 
            ST_SetSRID(ST_MakePoint(lat / 10000000.0, lon / 10000000.0), 4326) as geom
        FROM osm_2po_4pgr
        JOIN planet_osm_nodes as nodes
        ON osm_source_id = nodes.id
    )
    ORDER BY ST_Distance(
        geom,
        ST_SetSRID(ST_MakePoint($1, $2), 4326)
    )
    LIMIT 1;
"#;

async fn get_source_osm_id(
    client: &Client,
    Coordinates { lat, lon }: Coordinates,
) -> Result<i64, tokio_postgres::Error> {
    let stmt = client.prepare(SOURCE_OSM_SQL_QUERY).await?;

    let row = client.query_one(&stmt, &[&lat, &lon]).await?;

    Ok(row.get(0))
}

const TARGET_OSM_SQL_QUERY: &str = r#"
    SELECT osm_target_id
    FROM (
        SELECT DISTINCT 
            osm_target_id, 
            ST_SetSRID(ST_MakePoint(lat / 10000000.0, lon / 10000000.0), 4326) as geom
        FROM osm_2po_4pgr
        JOIN planet_osm_nodes as nodes
        ON osm_source_id = nodes.id
    )
    ORDER BY ST_Distance(
        geom,
        ST_SetSRID(ST_MakePoint($1, $2), 4326)
    )
    LIMIT 1;
"#;

async fn get_target_osm_id(
    client: &Client,
    Coordinates { lat, lon }: Coordinates,
) -> Result<i64, tokio_postgres::Error> {
    let stmt = client.prepare(TARGET_OSM_SQL_QUERY).await?;

    let row = client.query_one(&stmt, &[&lat, &lon]).await?;

    Ok(row.get(0))
}

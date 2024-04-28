use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AddressObject {
    number: String,
    street: String,
    city: String,
    state: String,
    country: String,
}

impl From<tokio_postgres::Row> for AddressObject {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            number: row.get("number"),
            street: row.get("street"),
            city: row.get("city"),
            state: row.get("state"),
            country: row.get("country"),
        }
    }
}

const ADDRESS_QUERY: &str = r#"
WITH point AS (
    SELECT ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857) AS geom
)
SELECT 
    "addr:housenumber" AS number,
    tags->'addr:street' AS street,
    tags->'addr:city' AS city,
    tags->'addr:state' AS state,
    COALESCE(tags->'addr:country', 'US') AS country
FROM 
    (
        SELECT "addr:housenumber", tags, way FROM planet_osm_line
        UNION ALL
        SELECT "addr:housenumber", tags, way FROM planet_osm_point
        UNION ALL
        SELECT "addr:housenumber", tags, way FROM planet_osm_polygon
        UNION ALL
        SELECT "addr:housenumber", tags, way FROM planet_osm_roads
    ) AS tables, point 
WHERE 
    "addr:housenumber" IS NOT NULL 
    AND tags ?& ARRAY['addr:street', 'addr:city', 'addr:state'] 
    AND ST_DWithin(way, point.geom, $3) 
ORDER BY 
    ST_Distance(way, point.geom) 
LIMIT 1;
"#;

const DIST_TOL: f64 = 1000.; // meters

pub async fn get_address(
    client: &tokio_postgres::Client,
    lat: f64,
    lon: f64,
    dist: Option<f64>,
) -> Result<AddressObject, tokio_postgres::Error> {
    let stmt = client.prepare(ADDRESS_QUERY).await?;

    let row = client
        .query_one(
            &stmt,
            &[&lon, &lat, &dist.unwrap_or(DIST_TOL)],
        )
        .await?;

    Ok(row.into())
}

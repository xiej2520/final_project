use crate::CONFIG;

pub struct DbClient {
    client: &'static tokio_postgres::Client,
    bbox_stmt: tokio_postgres::Statement,
    anywhere_stmt: tokio_postgres::Statement,
}
impl DbClient {
    pub async fn new() -> Result<Self, tokio_postgres::Error> {
        let (client, db_conn) = tokio_postgres::connect(CONFIG.db_url, tokio_postgres::NoTls)
            .await
            .expect("Failed to connect to postgresql server");
        tokio::spawn(async move {
            if let Err(e) = db_conn.await {
                tracing::error!("Postgres DB connection error: {e}");
            }
        });
        let client = Box::leak(Box::new(client));
        let bbox_stmt = client.prepare(BBOX_SQL_QUERY).await?;
        let anywhere_stmt = client.prepare(ANYWHERE_SQL_QUERY).await?;

        Ok(Self {
            client,
            bbox_stmt,
            anywhere_stmt,
        })
    }
    pub async fn anywhere_query(
        &self,
        search_term: &str,
    ) -> Result<Vec<tokio_postgres::Row>, tokio_postgres::Error> {
        self.client
            .query(&self.anywhere_stmt, &[&search_term])
            .await
    }
    pub async fn bbox_query(
        &self,
        search_term: &str,
        min_lon: f64,
        min_lat: f64,
        max_lon: f64,
        max_lat: f64,
    ) -> Result<Vec<tokio_postgres::Row>, tokio_postgres::Error> {
        self.client
            .query(
                &self.bbox_stmt,
                &[&min_lon, &min_lat, &max_lon, &max_lat, &search_term],
            )
            .await
    }
}

pub const BBOX_SQL_QUERY: &str = r#"
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
        (SELECT name, way FROM planet_osm_polygon
          WHERE name LIKE $5 AND ST_Intersects(way, (SELECT bbox FROM transformed_bbox))
          LIMIT 50
        ) UNION ALL
        (SELECT name, way FROM planet_osm_line
          WHERE name LIKE $5 AND ST_Intersects(way, (SELECT bbox FROM transformed_bbox))
          LIMIT 50
        ) UNION ALL
        (SELECT name, way FROM planet_osm_roads
          WHERE name LIKE $5 AND ST_Intersects(way, (SELECT bbox FROM transformed_bbox))
          LIMIT 50
        ) UNION ALL
        (SELECT name, way FROM planet_osm_point
          WHERE name LIKE $5 AND ST_Intersects(way, (SELECT bbox FROM transformed_bbox))
          LIMIT 50
        )
    ) AS relevant_tables, transformed_bbox
) AS intersection_query, transformed_bbox
ORDER BY 
    distance_to_center
LIMIT 50;
"#;

pub const ANYWHERE_SQL_QUERY: &str = r#"
SELECT 
    name,
    ST_Y(ST_Transform(ST_Centroid(way), 4326)) AS centroidLat,
    ST_X(ST_Transform(ST_Centroid(way), 4326)) AS centroidLon,
    ST_YMin(ST_Transform(ST_Envelope(way), 4326)) AS minLat,
    ST_XMin(ST_Transform(ST_Envelope(way), 4326)) AS minLon,
    ST_YMax(ST_Transform(ST_Envelope(way), 4326)) AS maxLat,
    ST_XMax(ST_Transform(ST_Envelope(way), 4326)) AS maxLon
FROM (
    (SELECT name, way FROM planet_osm_polygon WHERE name LIKE $1 LIMIT 50)
    UNION ALL
    (SELECT name, way FROM planet_osm_line WHERE name LIKE $1 LIMIT 50)
    UNION ALL
    (SELECT name, way FROM planet_osm_roads WHERE name LIKE $1 LIMIT 50)
    UNION ALL
    (SELECT name, way FROM planet_osm_point WHERE name LIKE $1 LIMIT 50)
) AS relevant_tables
LIMIT 50;
"#;

use std::f64::consts::PI;

use axum::body::Bytes;

use crate::CONFIG;

// rad -> (lat, lon)
// https://stackoverflow.com/questions/6671183/calculate-the-center-point-of-multiple-latitude-longitude-coordinate-pairs
fn center(tl_lat: f64, tl_lon: f64, br_lat: f64, br_lon: f64) -> (f64, f64) {
    let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);

    x += tl_lat.cos() * tl_lon.cos();
    y += tl_lat.cos() * tl_lon.sin();
    z += tl_lat.sin();

    x += br_lat.cos() * br_lon.cos();
    y += br_lat.cos() * br_lon.sin();
    z += br_lat.sin();

    x /= 2.0;
    y /= 2.0;
    z /= 2.0;

    // center
    let (lat, lon) = (f64::atan2(z, (x * x + y * y).sqrt()), f64::atan2(y, x));
    (lat, lon)
}

// rad -> (lat, lon)
#[allow(dead_code)]
fn center2(tl_lat: f64, tl_lon: f64, br_lat: f64, br_lon: f64) -> (f64, f64) {
    const T: f64 = 2.0 * PI;
    let ry1 = f64::ln((tl_lat.sin() + 1.0) / tl_lat.cos());
    let ry2 = f64::ln((br_lat.sin() + 1.0) / br_lat.cos());

    let ryc = (ry1 + ry2) / 2.0;
    let lat = ryc.sinh().atan();

    let lon1 = (tl_lon + T) % T;
    let lon2 = (br_lon + T) % T;
    let mut lon = (lon1 + lon2) / 2.0;
    if lon >= PI {
        // +180 exclusive
        lon -= T;
    } else if lon < -PI {
        // -180 inclusive
        lon += T;
    }
    (lat, lon)
}

pub async fn get_tile(tl_lat: f64, tl_lon: f64, br_lat: f64, br_lon: f64) -> Result<Bytes, reqwest::Error> {
    let (tl_lat, tl_lon, br_lat, br_lon) = (
        tl_lat.to_radians(),
        tl_lon.to_radians(),
        br_lat.to_radians(),
        br_lon.to_radians(),
    );
    let (lat, lon) = center(tl_lat, tl_lon, br_lat, br_lon);

    // zoom factor
    // https://gis.stackexchange.com/questions/19632/how-to-calculate-the-optimal-zoom-level-to-display-two-or-more-points-on-a-map
    let min_x = tl_lon.min(br_lon);
    let max_x = tl_lon.max(br_lon);

    let min_y = tl_lat.min(br_lat);
    let max_y = tl_lat.max(br_lat);
    dbg!(min_x, max_x, min_y, max_y);

    // lat -> mercator y
    let ry1 = f64::ln((min_y.sin() + 1.0) / min_y.cos());
    let ry2 = f64::ln((max_y.sin() + 1.0) / max_y.cos());
    let ryc = (ry1 + ry2) / 2.0;

    let center_y = ryc.sinh().atan().to_degrees();

    const VIEWPORT_WIDTH: f64 = 100.0; // idk what this is, assuming image width?
    let resolution_horizontal = (min_x - max_x).abs().to_degrees() / VIEWPORT_WIDTH;

    let vy0 = (PI * (0.25 + center_y / 360.0)).tan().ln();
    let vy1 = (PI * (0.25 + max_y.to_degrees() / 360.0)).tan().ln();

    const VIEWPORT_HEIGHT: f64 = 100.0; // assuming image height;
    const VIEW_HEIGHT_HALF: f64 = VIEWPORT_HEIGHT / 2.0;

    const EQUATOR_MM: f64 = 40.7436654315252;
    let zoom_factor_powered = VIEW_HEIGHT_HALF / (EQUATOR_MM * (vy1 - vy0));
    dbg!(vy0, vy1, zoom_factor_powered);

    let resolution_vertical = 360.0 / (zoom_factor_powered * 256.0);
    dbg!(resolution_horizontal, resolution_vertical);

    const PADDING_FACTOR: f64 = 1.2;
    let resolution = resolution_horizontal.max(resolution_vertical) * PADDING_FACTOR;

    let zoom = (360.0 / (resolution * 256.0)).log2();
    let zoom = zoom.clamp(0.0, 22.0);

    // http://localhost:8080/styles/osm-bright/static/-74,40.5,8.4/256x256@3x.png
    let (lat, lon) = (lat.to_degrees(), lon.to_degrees());
    let url = format!(
        "{}/{lon},{lat},{zoom}/100x100.png",
        CONFIG.tile_server_center_url
    );
    tracing::info!("{url}");

    let response = reqwest::get(url).await?;

    response.bytes().await
}

// localhost/turn/83.979259,-90.229003/0.865903,-65.359481.png negative zoom?

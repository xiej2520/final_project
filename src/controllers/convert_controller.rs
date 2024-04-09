use std::f64::consts::PI;

#[inline]
pub fn get_tile(lat: f64, lon: f64, zoom: f64) -> (i32, i32) {
    let n = (1 << (zoom as i32)) as f64;
    let x_tile = (n * (lon + 180.0) / 360.0) as i32; // round down is correct

    let lat_rad = lat.to_radians();
    let y_tile = (n * (1.0 - (lat_rad.tan() + (1.0 / lat_rad.cos())).ln() / PI) / 2.0) as i32;

    (x_tile, y_tile)
}

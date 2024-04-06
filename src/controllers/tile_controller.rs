use axum::body::Bytes;

use crate::CONFIG;

#[inline]
pub async fn get_tile(layer: i32, x: i32, y: i32) -> Bytes {
    let url = format!("{}{layer}/{x}/{y}.png", CONFIG.tile_server_url);

    let response = reqwest::get(url).await.unwrap();

    response.bytes().await.unwrap()
}

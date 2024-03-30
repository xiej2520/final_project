use axum::body::Bytes;

#[inline]
pub async fn get_tile(layer: i32, x: i32, y: i32) -> Bytes {
    let url = format!("http://127.0.0.1:8080/tile/{layer}/{x}/{y}.png");

    let response = reqwest::get(url).await.unwrap();

    response.bytes().await.unwrap()
}

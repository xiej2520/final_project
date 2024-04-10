use axum::body::Bytes;

use server::http_client::HttpClient;

#[inline]
pub async fn get_tile(
    client: &HttpClient,
    layer: i32,
    x: i32,
    y: i32,
) -> Result<Bytes, String> {
    let url = format!("{layer}/{x}/{y}.png");

    let builder = match client.get(&url).await {
        Ok(builder) => builder,
        Err(e) => return Err(e.to_string()),
    };
    let response = match builder.send().await {
        Ok(response) => response,
        Err(e) => return Err(e.to_string()),
    };

    match response.bytes().await {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(e.to_string()),
    }
}

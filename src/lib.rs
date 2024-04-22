pub mod controllers;
pub mod http_client;
pub mod parse_form;
pub mod routers;
pub mod status_response;

use axum::{
    body::{Body, Bytes},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum::{extract::Request, middleware::Next};
use chrono::Local;
use config::Config;
use http_body_util::BodyExt;
use once_cell::sync::Lazy;
use status_response::StatusResponse;
use tower_sessions::Session;
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, Layer};

#[derive(Debug)]
pub struct ServerConfig {
    pub ip: [u8; 4],
    pub http_port: u16,
    pub domain: &'static str,
    pub relay_ip: [u8; 4],
    pub relay_port: u16,
    pub search_url: &'static str,
    pub address_url: &'static str,
    pub tile_url: &'static str,
    pub turn_url: &'static str,
    pub routing_url: &'static str,
    pub submission_id: &'static str,
}

pub static CONFIG: Lazy<ServerConfig> = Lazy::new(|| {
    let config = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap();

    dbg!(ServerConfig {
        ip: config.get("ip").unwrap(),
        http_port: config.get("http_port").unwrap(),
        domain: config.get_string("domain").unwrap().leak(),
        relay_ip: config.get("relay_ip").unwrap(),
        relay_port: config.get("relay_port").unwrap(),
        // db_url: config.get_string("db_url").unwrap().leak(),
        search_url: config.get_string("search_url").unwrap().leak(),
        address_url: config.get_string("address_url").unwrap().leak(),
        tile_url: config.get_string("tile_url").unwrap().leak(),
        turn_url: config.get_string("turn_url").unwrap().leak(),
        routing_url: config.get_string("routing_url").unwrap().leak(),
        submission_id: config.get_string("submission_id").unwrap().leak(),
    })
});

pub async fn append_headers(req: Request, next: Next) -> Response<Body> {
    let mut res = next.run(req).await;
    res.headers_mut()
        .insert("x-cse356", CONFIG.submission_id.parse().unwrap());
    res
}

pub async fn login_gateway(req: Request, next: Next) -> Response {
    match req.extensions().get::<Session>() {
        Some(session) => match session.get::<String>("username").await {
            Ok(Some(_)) => next.run(req).await,
            _ => Json(StatusResponse::new_err("Unauthorized".to_owned())).into_response(),
        },
        None => Json(StatusResponse::new_err("Unauthorized".to_owned())).into_response(),
    }
}

pub async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<Response<Body>, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

pub async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        let count = body.len();
        tracing::debug!("{direction} body: {count} bytes = {body:?}");
    } else {
        tracing::debug!("{direction} body is not UTF-8");
    }

    Ok(bytes)
}

pub fn init_logging() {
    if cfg!(feature = "disable_logs") {
        return;
    }
    let log_appender = tracing_appender::rolling::never("./logs", Local::now().to_rfc3339());
    let log_format = tracing_subscriber::fmt::format::Format::default()
        //.with_target(true)
        //.with_level(true)
        //.with_thread_ids(true)
        //.with_thread_names(true)
        //.pretty()
        ;
    let local_log = tracing_subscriber::fmt::layer()
        .with_writer(log_appender)
        .with_writer(std::io::stderr)
        .event_format(log_format)
        //.with_filter(LevelFilter::DEBUG)
        //.with_filter(LevelFilter::INFO)
        .with_filter(LevelFilter::ERROR);
    //.with_filter(tracing_subscriber::filter::LevelFilter::ERROR);
    let registry = tracing_subscriber::registry().with(local_log);
    tracing::subscriber::set_global_default(registry).expect("Failed to enable logs");
}

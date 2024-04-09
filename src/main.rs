use std::{net::SocketAddr, sync::Arc};

use config::Config;
use once_cell::sync::Lazy;

use axum::{
    body::{Body, Bytes},
    http::StatusCode,
    response::Response,
    Router,
};
use axum::{extract::Request, middleware::Next};
use chrono::Local;

use http_body_util::BodyExt;

use tokio::sync::Mutex;
use tokio_postgres::NoTls;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer};

pub mod controllers;
pub mod routers;

use controllers::*;
use routers::*;

#[derive(Debug)]
struct ServerConfig {
    ip: [u8; 4],
    http_port: u16,
    domain: String,
    relay_ip: [u8; 4],
    relay_port: u16,
    db_url: String,
    tile_server_url: String,
    tile_server_center_url: String,
    routing_url: String,
    submission_id: String,
}

static CONFIG: Lazy<ServerConfig> = Lazy::new(|| {
    let config = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap();

    dbg!(ServerConfig {
        ip: config.get("ip").unwrap(),
        http_port: config.get("http_port").unwrap(),
        domain: config.get("domain").unwrap(),
        relay_ip: config.get("relay_ip").unwrap(),
        relay_port: config.get("relay_port").unwrap(),
        db_url: config.get("db_url").unwrap(),
        tile_server_url: config.get("tile_server_url").unwrap(),
        tile_server_center_url: config.get("tile_server_center_url").unwrap(),
        routing_url: config.get("routing_url").unwrap(),
        submission_id: config.get("submission_id").unwrap(),
    })
});

pub struct ServerState {
    client: tokio_postgres::Client,
}

impl ServerState {
    pub async fn new() -> Self {
        let (client, connection) = tokio_postgres::connect(&CONFIG.db_url, NoTls)
            .await
            .expect("Failed to connect to postgresql server");
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        Self { client }
    }
}

#[tokio::main]
async fn main() {
    let file_appender = tracing_appender::rolling::never("./logs", Local::now().to_rfc3339());
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish()
            .with(fmt::Layer::default().with_writer(file_writer)),
        // .with(fmt::Layer::default().with_writer(std::io::stderr))
    )
    .expect("Unable to set global tracing subscriber");

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600)));

    let user_store = Arc::new(Mutex::new(user_controller::UserStore::default()));
    let server_state = Arc::new(Mutex::new(ServerState::new().await));

    let app = Router::new()
        .nest_service("/", ServeDir::new("static"))
        .nest("/tiles", tile_router::new_router())
        .nest("/turn", turn_router::new_router())
        .nest(
            "/api",
            user_router::new_router().with_state(user_store.clone()),
        )
        .nest(
            "/api",
            search_router::new_router().with_state(server_state.clone()),
        )
        .nest("/", convert_router::new_router())
        .layer(axum::middleware::from_fn(append_headers))
        .layer(axum::middleware::from_fn(print_request_response))
        .layer(TraceLayer::new_for_http())
        .layer(session_layer);

    let addr = SocketAddr::from((CONFIG.ip, CONFIG.http_port));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::debug!("Server listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn append_headers(req: Request, next: Next) -> Response<Body> {
    let mut res = next.run(req).await;
    res.headers_mut()
        .insert("x-cse356", CONFIG.submission_id.parse().unwrap());
    res
}

async fn print_request_response(
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

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
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

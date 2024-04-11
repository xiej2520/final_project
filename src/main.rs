use std::{net::SocketAddr, sync::Arc};

use config::Config;
use once_cell::sync::Lazy;

use axum::{
    body::{Body, Bytes},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json, Router,
};
use axum::{extract::Request, middleware::Next};
use chrono::Local;

use http_body_util::BodyExt;

use tokio::sync::Mutex;
use tokio_postgres::NoTls;
use tower::ServiceBuilder;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, Session, SessionManagerLayer};

pub mod controllers;
pub mod routers;

use controllers::*;
use routers::*;
use server::{http_client::HttpClient, status_response::StatusResponse};

#[derive(Debug)]
struct ServerConfig {
    ip: [u8; 4],
    http_port: u16,
    domain: String,
    relay_ip: [u8; 4],
    relay_port: u16,
    db_url: String,
    tile_url: String,
    turn_url: String,
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
        tile_url: config.get("tile_url").unwrap(),
        turn_url: config.get("turn_url").unwrap(),
        routing_url: config.get("routing_url").unwrap(),
        submission_id: config.get("submission_id").unwrap(),
    })
});

pub struct ServerState {
    user_store: Arc<Mutex<user_controller::UserStore>>,
    db_client: Arc<tokio_postgres::Client>,
    // no need for Arc as reqwest::Client already implements it
    tile_client: HttpClient,
    turn_client: HttpClient,
    routing_client: HttpClient,
}

impl ServerState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let user_store = user_controller::UserStore::default();
        let (db_client, db_conn) = tokio_postgres::connect(&CONFIG.db_url, NoTls)
            .await
            .expect("Failed to connect to postgresql server");
        tokio::spawn(async move {
            if let Err(e) = db_conn.await {
                eprintln!("Connection error: {}", e);
            }
        });
        let tile_client = HttpClient::new(&CONFIG.tile_url)?;
        let turn_client = HttpClient::new(&CONFIG.turn_url)?;
        let routing_client = HttpClient::new(&CONFIG.routing_url)?;
        Ok(Self {
            user_store: Arc::new(Mutex::new(user_store)),
            db_client: Arc::new(db_client),
            tile_client,
            turn_client,
            routing_client,
        })
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

    let ServerState {
        user_store,
        db_client,
        tile_client,
        turn_client,
        routing_client,
    } = ServerState::new().await.expect("Something went wrong");

    let app = Router::new()
        .nest(
            "/api",
            search_router::new_router().with_state(db_client.clone()),
        )
        .nest(
            "/api",
            route_router::new_router().with_state(routing_client),
        )
        .nest("/", convert_router::new_router())
        .nest("/", tile_router::new_router().with_state(tile_client))
        .nest("/", turn_router::new_router().with_state(turn_client))
        .layer(axum::middleware::from_fn(login_gateway));

    let gateway = Router::new()
        .nest("/", app)
        .nest(
            "/api",
            user_router::new_router().with_state(user_store.clone()),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(print_request_response))
                .layer(axum::middleware::from_fn(append_headers)),
        ) 
        .nest_service("/", ServeDir::new("static"))
        .layer(session_layer); 

    let addr = SocketAddr::from((CONFIG.ip, CONFIG.http_port));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::debug!("Server listening on {}", addr);
    axum::serve(listener, gateway).await.unwrap();
}

async fn login_gateway(req: Request, next: Next) -> Response {
    match req.extensions().get::<Session>() {
        Some(session) => match session.get::<String>("username").await {
            Ok(Some(_)) => next.run(req).await,
            _ => Json(StatusResponse::new_err("Unauthorized".to_owned())).into_response(),
        },
        None => Json(StatusResponse::new_err("Unauthorized".to_owned())).into_response(),
    }
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

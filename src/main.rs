use std::net::SocketAddr;

use axum::{extract::Request, middleware::Next};
use axum::{
    response::{IntoResponse, Response},
    Json, Router,
};

use tokio::sync::RwLock;
use tokio_postgres::NoTls;

use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, Session, SessionManagerLayer};

use server::{
    config::CONFIG, controllers::*, http_client::HttpClient, logging::*, routers::*,
    status_response::StatusResponse,
};

pub struct ServerState {
    user_store: &'static RwLock<user_controller::UserStore>,
    db_client: &'static tokio_postgres::Client,
    // no need for Arc as reqwest::Client already implements it
    tile_client: HttpClient,
    turn_client: HttpClient,
    route_client: HttpClient,
}

impl ServerState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let user_store = Box::leak(Box::new(RwLock::new(user_controller::UserStore::new(
            CONFIG.domain,
            CONFIG.relay_ip,
            CONFIG.relay_port,
        ))));
        let (db_client, db_conn) = tokio_postgres::connect(CONFIG.db_url, NoTls)
            .await
            .expect("Failed to connect to postgresql server");
        let db_client = Box::leak(Box::new(db_client));
        tokio::spawn(async move {
            if let Err(e) = db_conn.await {
                panic!("Postgres DB connection error: {e}");
            }
        });
        let tile_client = HttpClient::new(CONFIG.tile_url)?;
        let turn_client = HttpClient::new(CONFIG.turn_url)?;
        let route_client = HttpClient::new(CONFIG.route_url)?;
        Ok(Self {
            user_store,
            db_client,
            tile_client,
            turn_client,
            route_client,
        })
    }
}

#[tokio::main]
async fn main() {
    init_logging();

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600)));

    let ServerState {
        user_store,
        db_client,
        tile_client,
        turn_client,
        route_client,
    } = ServerState::new()
        .await
        .map_err(|e| {
            tracing::error!("{e}");
        })
        .unwrap();

    let restricted_app = Router::new()
        .nest("/api", search_router::new_router().with_state(db_client))
        .nest("/api", address_router::new_router().with_state(db_client))
        .nest("/", convert_router::new_router())
        .nest(
            "/api",
            route_router::new_router().with_state(route_client),
        )
        .layer(axum::middleware::from_fn(login_gateway));

    let mut app = Router::new()
        .nest_service("/", ServeDir::new("static"))
        .nest("/api", user_router::new_router().with_state(user_store))
        .nest("/", tile_router::new_router().with_state(tile_client))
        .nest("/", turn_router::new_router().with_state(turn_client))
        .nest("/", restricted_app)
        .layer(session_layer);

    if !cfg!(feature = "disable_logs") {
        app = app
            .layer(axum::middleware::from_fn(print_request_response))
            .layer(TraceLayer::new_for_http());
    }

    let addr = SocketAddr::from((CONFIG.ip, CONFIG.http_port));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn login_gateway(req: Request, next: Next) -> Response {
    if cfg!(feature = "disable_auth") {
        return next.run(req).await;
    }

    match req.extensions().get::<Session>() {
        Some(session) => match session.get::<String>("username").await {
            Ok(Some(_)) => next.run(req).await,
            _ => Json(StatusResponse::new_err("Unauthorized".to_owned())).into_response(),
        },
        None => Json(StatusResponse::new_err("Unauthorized".to_owned())).into_response(),
    }
}

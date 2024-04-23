use std::net::SocketAddr;

use axum::{extract::Request, middleware::Next};
use axum::{
    response::{IntoResponse, Response},
    Json, Router,
};

use tokio::sync::RwLock;
use tower::ServiceBuilder;

use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, Session, SessionManagerLayer};

use server::routers::*;
use server::CONFIG;
use server::{controllers::*, init_logging, print_request_response};
use server::{http_client::HttpClient, StatusResponse};

pub struct ServerState {
    user_store: &'static RwLock<user_controller::UserStore>,
    // no need for Arc as reqwest::Client already implements it
    photon_client: HttpClient,
    nominatim_client: HttpClient,
    tile_client: HttpClient,
    turn_client: HttpClient,
    routing_client: HttpClient,
}

impl ServerState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let user_store = Box::leak(Box::new(RwLock::new(user_controller::UserStore::default())));
        let photon_client = HttpClient::new(CONFIG.photon_url)?;
        let nominatim_client = HttpClient::new(CONFIG.nominatim_url)?;
        let tile_client = HttpClient::new(CONFIG.tile_url)?;
        let turn_client = HttpClient::new(CONFIG.turn_url)?;
        let routing_client = HttpClient::new(CONFIG.routing_url)?;
        Ok(Self {
            user_store,
            photon_client,
            nominatim_client,
            tile_client,
            turn_client,
            routing_client,
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
        photon_client,
        nominatim_client,
        tile_client,
        turn_client,
        routing_client,
    } = ServerState::new()
        .await
        .map_err(|e| {
            tracing::error!("{e}");
        })
        .unwrap();

    let mut restricted_app = Router::new();
    if !cfg!(feature = "disable_auth") {
        restricted_app = restricted_app.layer(axum::middleware::from_fn(login_gateway));
    }

    let mut gateway = Router::new()
        .nest_service("/", ServeDir::new("static"))
        .nest("/auth", auth_router::new_router())
        .nest("/", convert_router::new_router())
        .nest("/", restricted_app)
        .nest("/api", user_router::new_router().with_state(user_store))
        .nest(
            "/api",
            search_router::new_router().with_state(photon_client.clone()),
        )
        .nest(
            "/",
            tile_router::new_router().with_state(tile_client.clone()),
        )
        .nest(
            "/",
            turn_router::new_router().with_state(turn_client.clone()),
        )
        .nest(
            "/api",
            address_router::new_router()
                .with_state((photon_client.clone(), nominatim_client.clone())),
        )
        .nest(
            "/api",
            route_router::new_router().with_state(routing_client.clone()),
        )
        .layer(session_layer)
        //.layer(axum::middleware::from_fn(append_headers))
        ;
    if !cfg!(feature = "disable_logs") {
        gateway = gateway.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(print_request_response)),
        );
    }

    let addr = SocketAddr::from((CONFIG.ip, CONFIG.http_port));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Server listening on {}", addr);
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

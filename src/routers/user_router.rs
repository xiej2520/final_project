use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_sessions::Session;

use crate::{controllers::user_controller::*, CONFIG};
use crate::status_response::StatusResponse;

#[derive(Debug, Clone, Deserialize)]
struct AddUserBody {
    username: String,
    password: String,
    email: String,
}

#[derive(Debug, Clone, Deserialize)]
struct VerifyParams {
    email: String,
    key: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LoginBody {
    username: String,
    password: String,
}

#[derive(Debug, Clone, Serialize)]
struct UserResponse {
    loggedin: bool,
    username: Option<String>,
}

pub fn new_router() -> Router<&'static RwLock<UserStore>> {
    Router::new()
        .route("/adduser", post(add_user_handler))
        .route("/verify", get(verify_user_handler))
        .route("/login", post(login_user_handler))
        .route("/logout", post(logout_user_handler))
        .route("/user", get(user_handler))
}

#[debug_handler]
async fn add_user_handler(
    State(store): State<&'static RwLock<UserStore>>,
    Json(AddUserBody {
        username,
        password,
        email,
    }): Json<AddUserBody>,
) -> Json<StatusResponse> {
    let user = User::new(&username, &password, &email);
    let mut store = store.write().await;
    match store.add_user(user, CONFIG.relay_ip, CONFIG.relay_port) {
        Ok(link) => Json(StatusResponse::new_ok(format!(
            "User added, verification url={link}",
        ))),
        Err(message) => Json(StatusResponse::new_err(message)),
    }
}

#[debug_handler]
async fn verify_user_handler(
    State(store): State<&'static RwLock<UserStore>>,
    Query(VerifyParams { email, key }): Query<VerifyParams>,
) -> Json<StatusResponse> {
    let mut store = store.write().await;
    match store.verify_user(&email, &key) {
        Ok(()) => Json(StatusResponse::new_ok("User verified".to_owned())),
        Err(message) => Json(StatusResponse::new_err(message)),
    }
}

#[debug_handler]
async fn login_user_handler(
    State(store): State<&'static RwLock<UserStore>>,
    session: Session,
    Json(LoginBody { username, password }): Json<LoginBody>,
) -> Json<StatusResponse> {
    if session.get::<String>("username").await.unwrap().is_some() {
        return Json(StatusResponse::new_ok("User already logged in".to_owned()));
    }

    let store = store.read().await;
    match store.get_user(&username) {
        Some(user) => {
            if user.is_enabled() && user.matches_password(&password) {
                session.insert("username", username).await.unwrap();
                Json(StatusResponse::new_ok("User logged in".to_owned()))
            } else if !user.is_enabled() {
                Json(StatusResponse::new_err("User not verified".to_owned()))
            } else {
                Json(StatusResponse::new_err("Invalid password".to_owned()))
            }
        }
        None => Json(StatusResponse::new_err("User not found".to_owned())),
    }
}

#[debug_handler]
async fn logout_user_handler(session: Session) -> Json<StatusResponse> {
    match session.get::<String>("username").await {
        Ok(Some(_)) => {
            session.remove::<String>("username").await.unwrap();
            Json(StatusResponse::new_ok("User logged out".to_owned()))
        }
        _ => Json(StatusResponse::new_err("User not logged in".to_owned())),
    }
}

#[debug_handler]
async fn user_handler(session: Session) -> Json<UserResponse> {
    match session.get::<String>("username").await {
        Ok(Some(username)) => Json(UserResponse {
            loggedin: true,
            username: Some(username),
        }),
        _ => Json(UserResponse {
            loggedin: false,
            username: None,
        }),
    }
}

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::Deserialize;
use tokio::sync::Mutex;
use tower_sessions::Session;

use crate::{controllers::user_controller::*, CONFIG};
use server::StatusResponse;

#[derive(Debug, Clone, Deserialize)]
struct AddUserBody {
    username: String,
    password: String,
    email: String,
}

#[derive(Debug, Clone, Deserialize)]
struct VerifyParams {
    email: Option<String>,
    key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LoginBody {
    username: String,
    password: String,
}

pub fn new_user_router() -> Router<Arc<Mutex<UserStore>>> {
    Router::new()
        .route("/adduser", post(add_user_handler))
        .route("/verify", get(verify_user_handler))
        .route("/login", post(login_user_handler))
        .route("/logout", post(logout_user_handler))
}

#[debug_handler]
async fn add_user_handler(
    State(store): State<Arc<Mutex<UserStore>>>,
    Json(AddUserBody {
        username,
        password,
        email,
    }): Json<AddUserBody>,
) -> (StatusCode, Json<StatusResponse>) {
    let mut store = store.lock().await;
    let (user, key) = User::new(&username, &password, &email);
    match store.add_user(user) {
        Ok(()) => match send_email(&email, &key).await {
            Ok(link) => (
                StatusCode::OK,
                Json(StatusResponse::new_ok(format!(
                    "User added, verification url={link}",
                ))),
            ),
            Err(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(StatusResponse::new_err(message)),
            ),
        },
        Err(message) => (
            StatusCode::BAD_REQUEST,
            Json(StatusResponse::new_err(message)),
        ),
    }
}

async fn send_email(email: &str, key: &str) -> Result<String, String> {
    // replace '+' in email with "%2b"
    let email = email.replace('+', "%2b");
    let verification_link = format!("http://{}/verify?email={email}&key={key}", CONFIG.domain);

    let email = Message::builder()
        .from(
            "warmup2 <warmup2@cse356.compas.cs.stonybrook.edu>"
                .parse()
                .unwrap(),
        )
        .to(email.parse().unwrap())
        .subject(verification_link.clone())
        .body(verification_link.clone())
        .unwrap();

    let relay_ip_string = CONFIG
        .relay_ip
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(".");
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&relay_ip_string)
        .port(CONFIG.relay_port)
        .build();
    match mailer.send(email).await {
        Ok(_) => Ok(verification_link),
        Err(err) => Err(format!("Failed to send email: {}", err)),
    }
}

#[debug_handler]
async fn verify_user_handler(
    State(store): State<Arc<Mutex<UserStore>>>,
    Query(VerifyParams { email, key }): Query<VerifyParams>,
) -> (StatusCode, Json<StatusResponse>) {
    let mut store = store.lock().await;
    let Some(email) = email else {
        return (
            StatusCode::BAD_REQUEST,
            Json(StatusResponse::new_err("Email not provided".to_owned())),
        );
    };
    let Some(key) = key else {
        return (
            StatusCode::BAD_REQUEST,
            Json(StatusResponse::new_err("Key not provided".to_owned())),
        );
    };
    match store.get_user_by_email_mut(&email) {
        Some(user) => match user.enable(&key) {
            Ok(()) => (
                StatusCode::OK,
                Json(StatusResponse::new_ok("User enabled".to_owned())),
            ),
            Err(message) => (
                StatusCode::BAD_REQUEST,
                Json(StatusResponse::new_err(message)),
            ),
        },
        None => (
            StatusCode::BAD_REQUEST,
            Json(StatusResponse::new_err("User not found".to_owned())),
        ),
    }
}

#[debug_handler]
async fn login_user_handler(
    State(store): State<Arc<Mutex<UserStore>>>,
    session: Session,
    Json(LoginBody { username, password }): Json<LoginBody>,
) -> (StatusCode, Json<StatusResponse>) {
    if session.get::<String>("username").await.unwrap().is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(StatusResponse::new_ok("User already logged in".to_owned())),
        );
    }

    let store = store.lock().await;
    match store.get_user(&username) {
        Some(user) => {
            if user.is_enabled() && user.matches_password(&password) {
                session.insert("username", username).await.unwrap();
                (
                    StatusCode::OK,
                    Json(StatusResponse::new_ok("User logged in".to_owned())),
                )
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    Json(StatusResponse::new_err(if !user.is_enabled() {
                        "User not verified".to_owned()
                    } else {
                        "Invalid password".to_owned()
                    })),
                )
            }
        }
        None => (
            StatusCode::BAD_REQUEST,
            Json(StatusResponse::new_err("User not found".to_owned())),
        ),
    }
}

#[debug_handler]
async fn logout_user_handler(session: Session) -> (StatusCode, Json<StatusResponse>) {
    match session.get::<String>("username").await {
        Ok(Some(_)) => {
            session.remove::<String>("username").await.unwrap();
            (
                StatusCode::OK,
                Json(StatusResponse::new_ok("User logged out".to_owned())),
            )
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(StatusResponse::new_err("User not logged in".to_owned())),
        ),
    }
}

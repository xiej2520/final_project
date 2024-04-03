use axum::{
    body::Body,
    extract::{FromRequest, Json, Request},
    Form,
};
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};

use serde::Serialize;

#[derive(Serialize)]
pub struct StatusResponse {
    pub message: String,
    pub status: &'static str,
}

impl StatusResponse {
    pub fn new_ok(message: String) -> Self {
        Self {
            status: "ok",
            message,
        }
    }
    pub fn new_err(message: String) -> Self {
        Self {
            status: "error",
            message,
        }
    }
}

pub async fn parse_form<T>(req: Request<Body>) -> Result<T, &'static str>
where
    Json<T>: FromRequest<()>,
    Form<T>: FromRequest<()>,
    T: TryFromMultipart,
{
    match req.headers().get("content-type") {
        Some(content_type) => {
            let content_type = content_type
                .to_str()
                .or(Err("Failed to parse content-type"))?;
            if content_type.contains("application/json") {
                match Json::<T>::from_request(req, &()).await {
                    Ok(Json(form)) => Ok(form),
                    Err(_) => Err("Failed to parse form"),
                }
            } else if content_type.contains("application/x-www-form-urlencoded") {
                match Form::<T>::from_request(req, &()).await {
                    Ok(Form(form)) => Ok(form),
                    Err(_) => Err("Failed to parse form"),
                }
            } else if content_type.contains("multipart/form-data") {
                match TypedMultipart::from_request(req, &()).await {
                    Ok(TypedMultipart::<T>(form)) => Ok(form),
                    Err(_) => Err("Failed to parse multipart"),
                }
            } else {
                Err("Unsupported content-type")
            }
        }
        None => Err("No content-type"),
    }
}

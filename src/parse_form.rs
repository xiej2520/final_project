use axum::{
    body::Body,
    extract::{FromRequest, Json, Request},
    Form,
};
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};

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

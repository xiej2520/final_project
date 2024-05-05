use axum::{
    body::{Body, Bytes},
    http::StatusCode,
    response::Response,
};
use axum::{extract::Request, middleware::Next};
use chrono::Local;
use http_body_util::BodyExt;
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, Layer};

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
        .event_format(log_format.clone())
        .with_filter(LevelFilter::DEBUG)
        //.with_filter(LevelFilter::INFO)
        //.with_filter(LevelFilter::ERROR);
        //.with_filter(tracing_subscriber::filter::LevelFilter::ERROR)
        ;
    let stderr_log = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .event_format(log_format)
        .with_filter(LevelFilter::DEBUG);
    let registry = tracing_subscriber::registry()
        .with(local_log)
        .with(stderr_log);
    tracing::subscriber::set_global_default(registry).expect("Failed to enable logs");
}

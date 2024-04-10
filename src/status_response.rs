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
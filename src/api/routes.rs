use axum::{Json, Router, routing::get};
use serde_json::json;

pub fn router() -> Router {
    Router::new().route("/", get(|| async { Json(json!({ "status": "ok" })) }))
}

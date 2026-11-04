use crate::api::dtos::BaseDto;
use crate::scopes;
use axum::Json;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
use serde_json::Value::Null;

pub fn router() -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async {
                (
                    StatusCode::OK,
                    Json(BaseDto {
                        message: "Healthy".to_string(),
                        data: serde_json::json!({"status":"ok"}),
                    }),
                )
            }),
        )
        .fallback(|| async {
            (
                StatusCode::NOT_FOUND,
                Json(BaseDto {
                    message: "Not Found".to_string(),
                    data: Null,
                }),
            )
        })
        .merge(scopes::routes::router())
}

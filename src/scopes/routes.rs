use crate::api::{dtos::BaseDto, impls::BaseImpl};
use crate::scopes::dtos::GetScopesDto;
use crate::scopes::impls::GetScopesUseCase;
use axum::{Json, Router, http::StatusCode, routing::get};

pub async fn get_scopes() -> (StatusCode, Json<BaseDto<GetScopesDto>>) {
    let use_case = GetScopesUseCase;
    let (status, response) = use_case.exec().await;
    (status, Json(response))
}

pub fn router() -> Router {
    Router::new().route("/scopes", get(get_scopes))
}

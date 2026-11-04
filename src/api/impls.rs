use crate::api::dtos::BaseDto;
use axum::http::StatusCode;
use serde::Serialize;

pub trait BaseImpl<T: Serialize> {
    async fn exec(&self) -> (StatusCode, BaseDto<T>);
}

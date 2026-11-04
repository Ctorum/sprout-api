use crate::api::{dtos, impls};
use crate::scopes::dtos::GetScopesDto;
use crate::scopes::enums::ScopesEnum;
use axum::http::StatusCode;

pub struct GetScopesUseCase;

impl impls::BaseImpl<GetScopesDto> for GetScopesUseCase {
    async fn exec(&self) -> (StatusCode, dtos::BaseDto<GetScopesDto>) {
        (
            StatusCode::OK,
            dtos::BaseDto {
                message: "Scopes retrieved succesfully".to_string(),
                data: GetScopesDto {
                    scopes: ScopesEnum::Personal,
                },
            },
        )
    }
}

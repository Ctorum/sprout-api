use crate::scopes::enums::ScopesEnum;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetScopesDto {
    pub scopes: ScopesEnum,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseDto<T: Serialize> {
    pub message: String,
    pub data: T,
}

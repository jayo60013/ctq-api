use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SolveLetterRequest {
    #[validate(custom(function = "crate::validators::validate_lowercase_letter"))]
    pub cipher_letter: char,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SolveLetterResponse {
    pub correct_letter: char,
}

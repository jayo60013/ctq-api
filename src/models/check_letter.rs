use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckLetterRequest {
    #[validate(custom(function = "crate::validators::validate_lowercase_letter"))]
    pub letter_to_check: char,

    #[validate(custom(function = "crate::validators::validate_lowercase_letter"))]
    pub cipher_letter: char,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckLetterResponse {
    pub is_letter_correct: bool,
}

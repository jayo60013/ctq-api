use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckQuoteRequest {
    pub cipher_map: HashMap<char, char>,
    pub checks_used: u16,
    pub solves_used: u16,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckQuoteResponse {
    pub is_quote_correct: bool,
}

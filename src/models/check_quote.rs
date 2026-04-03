use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CheckQuoteRequest {
    pub cipher_map: HashMap<char, char>,
    pub checks_used: u16,
    pub solves_used: u16,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckQuoteResponse {
    pub is_quote_correct: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streak: Option<i32>,
}

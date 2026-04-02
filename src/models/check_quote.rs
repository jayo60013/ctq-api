use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CheckQuoteRequest {
    pub cipher_map: HashMap<char, char>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckQuoteResponse {
    pub is_quote_correct: bool,
}

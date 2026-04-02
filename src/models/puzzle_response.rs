use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleResponse {
    pub id: i32,
    pub encoded_quote: String,
    pub author: String,
    pub source: Option<String>,
    pub date: NaiveDate,
}

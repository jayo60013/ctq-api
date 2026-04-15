use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleResponse {
    pub id: Uuid,
    pub encoded_quote: String,
    pub author: String,
    pub source: Option<String>,
    pub date: NaiveDate,
}

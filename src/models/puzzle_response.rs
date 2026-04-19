use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::PuzzleState;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleResponse {
    pub id: Uuid,
    pub encoded_quote: String,
    pub author: String,
    pub source: Option<String>,
    pub date: NaiveDate,
    pub state: PuzzleState,
}

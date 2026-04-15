use chrono::NaiveDate;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct PuzzleRow {
    pub id: Uuid,
    pub quote_id: i32,
    pub daily_date: NaiveDate,
    pub encoded_quote: String,
    pub cipher_map: serde_json::Value,
}

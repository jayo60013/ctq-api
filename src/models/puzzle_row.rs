use chrono::NaiveDate;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct PuzzleRow {
    pub id: i32,
    pub quote_id: i32,
    pub daily_date: NaiveDate,
    pub encoded_quote: String,
    pub cipher_map: serde_json::Value,
}

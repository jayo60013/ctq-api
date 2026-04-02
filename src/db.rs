use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{error::ApiError, models::puzzle_row::PuzzleRow, models::quote_row::QuoteRow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Puzzle {
    pub id: i32,
    pub quote_id: i32,
    pub daily_date: NaiveDate,
    pub encoded_quote: String,
    pub author: String,
    pub source: Option<String>,
    pub cipher_map: serde_json::Value,
}

pub struct PuzzleRepository {
    pool: PgPool,
}

impl PuzzleRepository {
    pub fn new(pool: PgPool) -> Self {
        PuzzleRepository { pool }
    }

    pub async fn get_by_id(&self, puzzle_id: i32) -> Result<Puzzle, ApiError> {
        let puzzle_row = sqlx::query_as::<_, PuzzleRow>(
            "SELECT id, quote_id, daily_date, encoded_quote, cipher_map FROM puzzles WHERE id = $1",
        )
        .bind(puzzle_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .ok_or(ApiError::NotFound)?;

        let quote_row =
            sqlx::query_as::<_, QuoteRow>("SELECT author, source FROM quotes WHERE id = $1")
                .bind(puzzle_row.quote_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(Puzzle {
            id: puzzle_row.id,
            quote_id: puzzle_row.quote_id,
            daily_date: puzzle_row.daily_date,
            encoded_quote: puzzle_row.encoded_quote,
            author: quote_row.author,
            source: quote_row.source,
            cipher_map: puzzle_row.cipher_map,
        })
    }
}

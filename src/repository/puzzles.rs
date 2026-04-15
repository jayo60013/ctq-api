use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::puzzle_row::PuzzleRow;
use crate::models::quote_row::QuoteRow;
use crate::transformer::parse_cipher_map_from_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Puzzle {
    pub id: Uuid,
    pub quote_id: i32,
    pub daily_date: NaiveDate,
    pub encoded_quote: String,
    pub author: String,
    pub source: Option<String>,
    pub cipher_map: HashMap<char, char>,
}

pub struct PuzzleRepository {
    pool: PgPool,
}

impl PuzzleRepository {
    pub fn new(pool: PgPool) -> Self {
        PuzzleRepository { pool }
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Puzzle, ApiError> {
        let puzzle_row = sqlx::query_as::<_, PuzzleRow>(
            r"
            SELECT id, quote_id, daily_date, encoded_quote, cipher_map
            FROM puzzles
            WHERE id = $1
            ",
        )
        .bind(id)
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
            cipher_map: parse_cipher_map_from_json(&puzzle_row.cipher_map)?,
        })
    }

    pub async fn get_by_date(&self, date: NaiveDate) -> Result<Puzzle, ApiError> {
        let puzzle_row = sqlx::query_as::<_, PuzzleRow>(
            r"
            SELECT id, quote_id, daily_date, encoded_quote, cipher_map
            FROM puzzles
            WHERE daily_date = $1
            ",
        )
        .bind(date)
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
            cipher_map: parse_cipher_map_from_json(&puzzle_row.cipher_map)?,
        })
    }
}

use chrono::{Local, NaiveDate};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    config::Config,
    db::{Puzzle, PuzzleRepository},
    error::ApiError,
    models::PuzzleResponse,
};

pub struct DailyPuzzleCache {
    response: Arc<RwLock<Option<CachedPuzzle>>>,
    last_date: Arc<RwLock<NaiveDate>>,
}

impl Default for DailyPuzzleCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct CachedPuzzle {
    response: PuzzleResponse,
    full_puzzle: Puzzle,
    date: NaiveDate,
}

impl DailyPuzzleCache {
    pub fn new() -> Self {
        Self {
            response: Arc::new(RwLock::new(None)),
            last_date: Arc::new(RwLock::new(Local::now().date_naive())),
        }
    }

    pub async fn get_response(
        &self,
        pool_repo: &PuzzleRepository,
        config: &Config,
    ) -> Result<PuzzleResponse, ApiError> {
        self.ensure_cached(pool_repo, config).await?;

        let cached = self.response.read().await;
        Ok(cached
            .as_ref()
            .expect("Cache should be populated")
            .response
            .clone())
    }

    pub async fn get_puzzle(
        &self,
        pool_repo: &PuzzleRepository,
        config: &Config,
    ) -> Result<Puzzle, ApiError> {
        self.ensure_cached(pool_repo, config).await?;

        let cached = self.response.read().await;
        Ok(cached
            .as_ref()
            .expect("Cache should be populated")
            .full_puzzle
            .clone())
    }

    async fn ensure_cached(
        &self,
        pool_repo: &PuzzleRepository,
        config: &Config,
    ) -> Result<(), ApiError> {
        let today = Local::now().date_naive();
        let mut last_date = self.last_date.write().await;

        if today != *last_date {
            let mut response = self.response.write().await;
            *response = None;
            *last_date = today;
        }
        drop(last_date);

        {
            let response = self.response.read().await;
            if let Some(cached) = response.as_ref()
                && cached.date == today
            {
                return Ok(());
            }
        }

        let puzzle_id = Self::calculate_puzzle_id(today, config);
        let db_puzzle = pool_repo
            .get_by_id(puzzle_id)
            .await
            .map_err(|err| match err {
                ApiError::NotFound => {
                    ApiError::DatabaseError("Puzzle not generated yet".to_string())
                }
                other => other,
            })?;

        let response = PuzzleResponse {
            id: db_puzzle.id,
            encoded_quote: db_puzzle.encoded_quote.clone(),
            author: db_puzzle.author.clone(),
            source: db_puzzle.source.clone(),
            date: db_puzzle.daily_date,
        };

        {
            let mut cached = self.response.write().await;
            *cached = Some(CachedPuzzle {
                response,
                full_puzzle: db_puzzle,
                date: today,
            });
        }

        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    fn calculate_puzzle_id(date: NaiveDate, config: &Config) -> i32 {
        let days_since_start = (date - config.start_date).num_days();
        (days_since_start + 1) as i32
    }
}

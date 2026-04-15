use chrono::{NaiveDate, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    error::ApiError,
    repository::{Puzzle, PuzzleRepository},
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
    full_puzzle: Puzzle,
    date: NaiveDate,
}

impl DailyPuzzleCache {
    pub fn new() -> Self {
        Self {
            response: Arc::new(RwLock::new(None)),
            last_date: Arc::new(RwLock::new(Utc::now().date_naive())),
        }
    }

    pub async fn get_puzzle(&self, pool_repo: &PuzzleRepository) -> Result<Puzzle, ApiError> {
        self.ensure_cached(pool_repo).await?;

        let cached = self.response.read().await;
        Ok(cached
            .as_ref()
            .expect("Cache should be populated")
            .full_puzzle
            .clone())
    }

    async fn ensure_cached(&self, pool_repo: &PuzzleRepository) -> Result<(), ApiError> {
        let today = Utc::now().date_naive();
        let mut last_date = self.last_date.write().await;

        if today != *last_date {
            let mut response = self.response.write().await;
            *response = None;
            *last_date = today;
        }
        drop(last_date);

        {
            let response = self.response.read().await;
            if let Some(cached) = response.as_ref() {
                if cached.date == today {
                    return Ok(());
                }
            }
        }

        let db_puzzle = pool_repo
            .get_by_date(today)
            .await
            .map_err(|err| match err {
                ApiError::NotFound => {
                    ApiError::DatabaseError("Puzzle not generated yet".to_string())
                }
                other => other,
            })?;

        {
            let mut cached = self.response.write().await;
            *cached = Some(CachedPuzzle {
                full_puzzle: db_puzzle,
                date: today,
            });
        }

        Ok(())
    }
}

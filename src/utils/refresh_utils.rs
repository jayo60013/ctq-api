use std::time::Duration;

use time::OffsetDateTime;

use crate::{
    DailyPuzzleCache, DailyPuzzleResponseCache, models::daily_puzzle_response::DailyPuzzleResponse,
    utils::daily_puzzle_utils::get_daily_puzzle_entity,
};
use log::{error, info};

use tokio::time::{Instant, MissedTickBehavior, interval_at};

pub fn spawn_daily_puzzle_refresh(
    pool: sqlx::PgPool,
    puzzle_cache: DailyPuzzleCache,
    response_cache: DailyPuzzleResponseCache,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) = refresh_once(&pool, &puzzle_cache, &response_cache).await {
            error!("Unable to refresh puzzle: {e}");
            std::process::exit(1);
        }

        let start = next_midnight_utc_instant();
        let mut interval = interval_at(start, Duration::from_secs(24 * 60 * 60));

        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            interval.tick().await;

            match refresh_once(&pool, &puzzle_cache, &response_cache).await {
                Ok(()) => {
                    info!("Daily puzzle updated successfully at midnight.");
                }
                Err(e) => {
                    error!("Unable to refresh puzzle: {e}");
                    std::process::exit(1);
                }
            }
        }
    })
}

async fn refresh_once(
    pool: &sqlx::PgPool,
    puzzle_cache: &DailyPuzzleCache,
    response_cache: &DailyPuzzleResponseCache,
) -> Result<(), anyhow::Error> {
    let new_puzzle = get_daily_puzzle_entity(pool).await?;

    let new_response = DailyPuzzleResponse {
        author: new_puzzle.author.clone(),
        source: new_puzzle.source.clone(),
        cipher_quote: new_puzzle.cipher_quote.clone(),
        date_string: new_puzzle.date_string.clone(),
        day_number: new_puzzle.day_number,
    };

    // If these two caches must change atomically, consider grouping them in one RwLock.
    *puzzle_cache.write().await = new_puzzle;
    *response_cache.write().await = new_response;

    Ok(())
}

fn next_midnight_utc_instant() -> Instant {
    let now = OffsetDateTime::now_utc();
    let next_midnight = now
        .date()
        .next_day()
        .expect("date overflow")
        .midnight()
        .assume_utc();
    let dur = (next_midnight - now)
        .try_into()
        .unwrap_or(Duration::from_secs(0));
    Instant::now() + dur
}

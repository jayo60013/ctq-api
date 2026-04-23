use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::{
    ActivityState, ActivitySummaryResponse, CheckQuoteState, Game, GlobalStats, PlayerStats,
};
use crate::repository::{
    get_activity, get_assist_budget_distribution, get_average_score, get_current_streak,
    get_highest_streak, get_puzzle_global_stats, get_puzzle_percentile,
    get_puzzles_with_activities_by_date_range, get_total_solved_puzzles, is_puzzle_solved,
    update_puzzle_global_stats, upsert_activity,
};
use crate::transformer::build_score_distribution;

pub struct ActivityService;

impl ActivityService {
    pub async fn fetch_activity_summary(
        pool: &PgPool,
        user_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<Vec<ActivitySummaryResponse>, ApiError> {
        let puzzles_with_activities =
            get_puzzles_with_activities_by_date_range(pool, user_id, from_date, to_date).await?;

        let response = puzzles_with_activities
            .into_iter()
            .map(|(daily_date, puzzle_id, activity)| {
                let state = activity.map(|act| ActivityState {
                    completed_at: act.completed_at,
                    checks_used: act.checks_used,
                    solves_used: act.solves_used,
                    is_solved: act.is_solved,
                    is_daily_flag: act.is_daily_flag,
                    current_streak: act.current_streak,
                    assist_budget: act.assist_budget,
                });

                ActivitySummaryResponse {
                    puzzle_id,
                    daily_date,
                    state,
                }
            })
            .collect();

        Ok(response)
    }

    /// Builds player stats (streaks, average, distribution) for a specific user
    pub async fn build_player_stats(pool: &PgPool, user_id: Uuid) -> Result<PlayerStats, ApiError> {
        let current_streak = get_current_streak(pool, user_id).await?;
        let best_streak = get_highest_streak(pool, user_id).await?;
        let average_score = get_average_score(pool, user_id).await?;
        let distribution_data = get_assist_budget_distribution(pool, user_id).await?;
        let total_solved = get_total_solved_puzzles(pool, user_id).await?;

        let distribution = build_score_distribution(&distribution_data, total_solved);

        Ok(PlayerStats {
            current_streak,
            best_streak,
            average_score,
            total_puzzles_completed: total_solved,
            distribution,
        })
    }

    /// Builds global stats for a puzzle based on user's score
    pub async fn build_global_stats(
        pool: &PgPool,
        puzzle_id: Uuid,
        user_score: i32,
    ) -> Result<Option<GlobalStats>, ApiError> {
        if let Some((solve_count, total_score_sum, score_distribution)) =
            get_puzzle_global_stats(pool, puzzle_id).await?
        {
            let average_score = if solve_count > 0 {
                #[allow(clippy::cast_precision_loss)]
                {
                    total_score_sum as f64 / solve_count as f64
                }
            } else {
                0.0
            };

            let distribution = score_distribution
                .iter()
                .enumerate()
                .map(|(idx, count)| {
                    let percentage = if solve_count > 0 {
                        #[allow(clippy::cast_precision_loss)]
                        {
                            (*count as f64 / solve_count as f64) * 100.0
                        }
                    } else {
                        0.0
                    };
                    crate::models::GlobalStatsBucket {
                        score: idx.to_string(),
                        percentage,
                    }
                })
                .collect();

            let percentile = get_puzzle_percentile(pool, puzzle_id, user_score).await?;

            Ok(Some(GlobalStats {
                average_score,
                distribution,
                percentile,
            }))
        } else {
            Ok(None)
        }
    }

    /// Records a puzzle solution: marks as solved, updates activity, and returns stats
    /// This is for daily puzzles (`is_daily_flag=true`)
    pub async fn record_solution(
        pool: &PgPool,
        user_id: Uuid,
        puzzle_id: Uuid,
    ) -> Result<CheckQuoteState, ApiError> {
        // Check if this was previously solved
        let was_already_solved = is_puzzle_solved(pool, user_id, puzzle_id)
            .await
            .unwrap_or(false);

        // Get current activity to get current stats
        let activity = get_activity(pool, user_id, puzzle_id).await?;
        let (checks_used, solves_used) = if let Some(a) = activity {
            (a.checks_used, a.solves_used)
        } else {
            (0, 0)
        };

        // Mark puzzle as solved
        upsert_activity(
            pool,
            user_id,
            puzzle_id,
            checks_used,
            solves_used,
            true,
            true,
        )
        .await?;

        // Calculate score
        let score = 6 - (checks_used + (solves_used * 2));

        // Update global stats only on first-time completion
        if !was_already_solved {
            update_puzzle_global_stats(pool, puzzle_id, score).await?;
        }

        // Build player and global stats
        let player = Self::build_player_stats(pool, user_id).await?;
        let global = Self::build_global_stats(pool, puzzle_id, score).await?;

        let game = Game {
            score,
            checks_used,
            solves_used,
        };

        let state = CheckQuoteState {
            game: Some(game),
            player,
            global,
        };

        Ok(state)
    }

    /// Records an archive puzzle solution (`is_daily_flag=false`)
    /// Same as `record_solution` but for archive puzzles
    pub async fn record_archive_solution(
        pool: &PgPool,
        user_id: Uuid,
        puzzle_id: Uuid,
    ) -> Result<CheckQuoteState, ApiError> {
        // Check if this was previously solved
        let was_already_solved = is_puzzle_solved(pool, user_id, puzzle_id)
            .await
            .unwrap_or(false);

        // Get current activity to get current stats
        let activity = get_activity(pool, user_id, puzzle_id).await?;
        let (checks_used, solves_used) = if let Some(a) = activity {
            (a.checks_used, a.solves_used)
        } else {
            (0, 0)
        };

        // Mark puzzle as solved (is_daily_flag=false for archive)
        upsert_activity(
            pool,
            user_id,
            puzzle_id,
            checks_used,
            solves_used,
            true,
            false, // Archive puzzles are not daily
        )
        .await?;

        // Calculate score
        let score = 6 - (checks_used + (solves_used * 2));

        // Update global stats only on first-time completion
        if !was_already_solved {
            update_puzzle_global_stats(pool, puzzle_id, score).await?;
        }

        // Build player and global stats
        let player = Self::build_player_stats(pool, user_id).await?;
        let global = Self::build_global_stats(pool, puzzle_id, score).await?;

        let game = Game {
            score,
            checks_used,
            solves_used,
        };

        let state = CheckQuoteState {
            game: Some(game),
            player,
            global,
        };

        Ok(state)
    }
}

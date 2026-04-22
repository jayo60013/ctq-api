use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::{
    ActivityState, ActivitySummaryResponse, ScoreDistributionBucket, ScoreRange, StatsResponse,
};
use crate::repository::{
    get_assist_budget_distribution, get_current_streak, get_highest_streak,
    get_puzzles_with_activities_by_date_range, get_total_played_puzzles,
};

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

    #[allow(clippy::cast_precision_loss)]
    pub async fn get_stats(pool: &PgPool, user_id: Uuid) -> Result<StatsResponse, ApiError> {
        let total_played_puzzles = get_total_played_puzzles(pool, user_id).await?;
        let current_streak = get_current_streak(pool, user_id).await?;
        let highest_streak = get_highest_streak(pool, user_id).await?;

        // Get the score distribution for solved puzzles
        let distribution = get_assist_budget_distribution(pool, user_id).await?;

        // Calculate total solved puzzles and build distribution buckets
        let total_solved: i64 = distribution.iter().map(|(_, _, count)| count).sum();
        let score_distribution = distribution
            .into_iter()
            .map(|(min, max, count)| {
                let percentage = if total_solved > 0 {
                    (count as f64 / total_solved as f64) * 100.0
                } else {
                    0.0
                };
                ScoreDistributionBucket {
                    range: ScoreRange { min, max },
                    count,
                    percentage: (percentage * 10.0).round() / 10.0, // Round to 1 decimal place
                }
            })
            .collect();

        Ok(StatsResponse {
            total_played_puzzles,
            current_streak,
            highest_streak,
            score_distribution,
        })
    }
}

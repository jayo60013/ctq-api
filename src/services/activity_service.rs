use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::{
    ActivityRow, ActivityState, ActivitySummaryResponse, ActivityUpdateRequest, StatsResponse,
};
use crate::repository::{
    get_activity, get_average_attempts, get_current_streak, get_highest_streak,
    get_puzzles_with_activities_by_date_range, get_total_played_puzzles, upsert_activity,
};

pub struct ActivityService;

impl ActivityService {
    pub async fn track_activity(
        pool: &PgPool,
        user_id: Uuid,
        puzzle_id: Uuid,
        is_solved: bool,
        is_daily_flag: bool,
        activity_request: &ActivityUpdateRequest,
    ) -> Result<i32, ApiError> {
        upsert_activity(
            pool,
            user_id,
            puzzle_id,
            i32::from(activity_request.checks_used),
            i32::from(activity_request.solves_used),
            is_solved,
            is_daily_flag,
        )
        .await?;

        let activity: ActivityRow =
            get_activity(pool, user_id, puzzle_id)
                .await?
                .ok_or_else(|| {
                    ApiError::DatabaseError("Activity not found after insert".to_string())
                })?;

        Ok(activity.current_streak)
    }

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
                    attempts: act.attempts,
                    checks_used: act.checks_used,
                    solves_used: act.solves_used,
                    is_solved: act.is_solved,
                    is_daily_flag: act.is_daily_flag,
                    current_streak: act.current_streak,
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

    pub async fn get_stats(pool: &PgPool, user_id: Uuid) -> Result<StatsResponse, ApiError> {
        let total_played_puzzles = get_total_played_puzzles(pool, user_id).await?;
        let current_streak = get_current_streak(pool, user_id).await?;
        let highest_streak = get_highest_streak(pool, user_id).await?;
        let average_attempts = get_average_attempts(pool, user_id).await?;

        Ok(StatsResponse {
            total_played_puzzles,
            current_streak,
            highest_streak,
            average_attempts,
        })
    }
}

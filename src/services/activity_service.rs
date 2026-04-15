use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::{ActivityRow, ActivityRowDto, ActivityUpdateRequest};
use crate::repository::{get_activities_by_date_range, get_activity, upsert_activity};
use crate::validators::validate_activity_request;

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
        validate_activity_request(activity_request)?;

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
    ) -> Result<Vec<ActivityRowDto>, ApiError> {
        let activities = get_activities_by_date_range(pool, user_id, from_date, to_date).await?;

        let response = activities.into_iter().map(ActivityRowDto::from).collect();

        Ok(response)
    }
}

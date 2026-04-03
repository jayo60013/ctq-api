use chrono::Local;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::EnvConfig;
use crate::error::ApiError;
use crate::models::{ActivityRow, ActivityUpdateRequest};
use crate::repository::upsert_activity;
use crate::repository::get_activity;
use crate::validators::validate_activity_request;

pub struct ActivityService;

impl ActivityService {
    pub async fn track_activity(
        pool: &PgPool,
        user_id: Uuid,
        puzzle_id: i32,
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

        let activity: ActivityRow = get_activity(pool, user_id, puzzle_id)
            .await?
            .ok_or_else(|| ApiError::DatabaseError("Activity not found after insert".to_string()))?;

        Ok(activity.current_streak)
    }

    pub fn calculate_puzzle_id_for_daily(config: &EnvConfig) -> i32 {
        let today = Local::now().date_naive();
        let days_since_start = (today - config.start_date).num_days();
        i32::try_from(days_since_start + 1).unwrap()
    }
}

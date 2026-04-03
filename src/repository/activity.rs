use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::EnvConfig;
use crate::error::ApiError;
use crate::models::ActivityRow;

pub async fn upsert_activity(
    pool: &PgPool,
    user_id: Uuid,
    puzzle_id: i32,
    checks_used: i32,
    solves_used: i32,
    is_solved: bool,
    is_daily_flag: bool,
) -> Result<(), ApiError> {
    let yesterday_puzzle_id = puzzle_id - 1;

    let current_streak = if is_daily_flag && is_solved {
        let yesterday_streak = sqlx::query_scalar::<_, i32>(
            "SELECT current_streak FROM user_puzzle_activity WHERE user_id = $1 AND puzzle_id = $2 AND is_solved = true"
        )
        .bind(user_id)
        .bind(yesterday_puzzle_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        match yesterday_streak {
            Some(streak) => streak + 1,
            None => 1,
        }
    } else {
        0
    };

    sqlx::query(
        r"
        INSERT INTO activities (user_id, puzzle_id, attempts, checks_used, solves_used, is_solved, is_daily_flag, completed_at, current_streak)
        VALUES ($1, $2, 1, $3, $4, $5, $6, CASE WHEN $5 = true THEN now() ELSE NULL END, $7)
        ON CONFLICT (user_id, puzzle_id)
        DO UPDATE SET
            attempts = activities.attempts + 1,
            checks_used = EXCLUDED.checks_used,
            solves_used = EXCLUDED.solves_used,
            is_solved = EXCLUDED.is_solved,
            is_daily_flag = EXCLUDED.is_daily_flag,
            current_streak = EXCLUDED.current_streak,
            completed_at = CASE
                WHEN EXCLUDED.is_solved = true THEN now()
                ELSE activities.completed_at
            END
        ",
    )
    .bind(user_id)
    .bind(puzzle_id)
    .bind(checks_used)
    .bind(solves_used)
    .bind(is_solved)
    .bind(is_daily_flag)
    .bind(current_streak)
    .execute(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub async fn get_activity(
    pool: &PgPool,
    user_id: Uuid,
    puzzle_id: i32,
) -> Result<Option<ActivityRow>, ApiError> {
    let activity = sqlx::query_as::<_, ActivityRow>(
        "SELECT user_id, puzzle_id, completed_at, attempts, checks_used, solves_used, is_solved, is_daily_flag, current_streak FROM user_puzzle_activity WHERE user_id = $1 AND puzzle_id = $2",
    )
    .bind(user_id)
    .bind(puzzle_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(activity)
}

pub async fn get_activities_by_date_range(
    pool: &PgPool,
    config: &EnvConfig,
    user_id: Uuid,
    from_date: NaiveDate,
    to_date: NaiveDate,
) -> Result<Vec<ActivityRow>, ApiError> {
    let days_from_start = i32::try_from((from_date - config.start_date).num_days()).unwrap();
    let days_to_start = i32::try_from((to_date - config.start_date).num_days()).unwrap();

    let from_puzzle_id = days_from_start + 1;
    let to_puzzle_id = days_to_start + 1;

    let activities = sqlx::query_as::<_, ActivityRow>(
        r"
        SELECT user_id, puzzle_id, completed_at, attempts, checks_used, solves_used, is_solved, is_daily_flag, current_streak
        FROM activities
        WHERE user_id = $1 AND puzzle_id >= $2 AND puzzle_id <= $3
        ORDER BY puzzle_id ASC
        "
    )
    .bind(user_id)
    .bind(from_puzzle_id)
    .bind(to_puzzle_id)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(activities)
}

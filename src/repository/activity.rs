use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::ActivityRow;

pub async fn upsert_activity(
    pool: &PgPool,
    user_id: Uuid,
    puzzle_id: Uuid,
    checks_used: i32,
    solves_used: i32,
    is_solved: bool,
    is_daily_flag: bool,
) -> Result<(), ApiError> {
    let current_streak = if is_daily_flag && is_solved {
        let yesterday_date = sqlx::query_scalar::<_, Option<NaiveDate>>(
            r"
            SELECT p.daily_date
            FROM activities a
            JOIN puzzles p ON a.puzzle_id = p.id
            WHERE a.user_id = $1
                AND p.daily_date < (SELECT daily_date FROM puzzles WHERE id = $2)
                AND a.is_solved = true
            ORDER BY p.daily_date DESC LIMIT 1
            ",
        )
        .bind(user_id)
        .bind(puzzle_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        if let Some(yesterday) = yesterday_date {
            let yesterday_streak = sqlx::query_scalar::<_, i32>(
                r"
                SELECT current_streak
                FROM activities a
                JOIN puzzles p ON a.puzzle_id = p.id
                WHERE a.user_id = $1
                    AND p.daily_date = $2
                    AND a.is_solved = true
                ",
            )
            .bind(user_id)
            .bind(yesterday)
            .fetch_optional(pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

            match yesterday_streak {
                Some(streak) => streak + 1,
                None => 1,
            }
        } else {
            1
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
    puzzle_id: Uuid,
) -> Result<Option<ActivityRow>, ApiError> {
    let activity = sqlx::query_as::<_, ActivityRow>(
        r"
        SELECT user_id, puzzle_id, completed_at, attempts, checks_used, solves_used, is_solved, is_daily_flag, current_streak
        FROM activities
        WHERE user_id = $1
            AND puzzle_id = $2
        "
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
    user_id: Uuid,
    from_date: NaiveDate,
    to_date: NaiveDate,
) -> Result<Vec<(ActivityRow, NaiveDate)>, ApiError> {
    #[derive(sqlx::FromRow)]
    struct ActivityWithDate {
        user_id: Uuid,
        puzzle_id: Uuid,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
        attempts: i32,
        checks_used: i32,
        solves_used: i32,
        is_solved: bool,
        is_daily_flag: bool,
        current_streak: i32,
        daily_date: NaiveDate,
    }

    let rows = sqlx::query_as::<_, ActivityWithDate>(
        r"
        SELECT a.user_id, a.puzzle_id, a.completed_at, a.attempts, a.checks_used, a.solves_used, a.is_solved, a.is_daily_flag, a.current_streak, p.daily_date
        FROM activities a
        JOIN puzzles p ON a.puzzle_id = p.id
        WHERE a.user_id = $1
            AND p.daily_date >= $2
            AND p.daily_date <= $3
        ORDER BY p.daily_date ASC
        "
    )
    .bind(user_id)
    .bind(from_date)
    .bind(to_date)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let activities = rows
        .into_iter()
        .map(|row| {
            (
                ActivityRow {
                    user_id: row.user_id,
                    puzzle_id: row.puzzle_id,
                    completed_at: row.completed_at,
                    attempts: row.attempts,
                    checks_used: row.checks_used,
                    solves_used: row.solves_used,
                    is_solved: row.is_solved,
                    is_daily_flag: row.is_daily_flag,
                    current_streak: row.current_streak,
                },
                row.daily_date,
            )
        })
        .collect();

    Ok(activities)
}

pub async fn get_total_played_puzzles(pool: &PgPool, user_id: Uuid) -> Result<i64, ApiError> {
    let count = sqlx::query_scalar::<_, i64>(
        r"
        SELECT COUNT(*) as total
        FROM activities
        WHERE user_id = $1
        ",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(count)
}

pub async fn get_current_streak(pool: &PgPool, user_id: Uuid) -> Result<i32, ApiError> {
    let streak = sqlx::query_scalar::<_, Option<i32>>(
        r"
        SELECT a.current_streak
        FROM activities a
        JOIN puzzles p ON a.puzzle_id = p.id
        WHERE a.user_id = $1
            AND a.is_daily_flag = true
            AND a.is_solved = true
        ORDER BY p.daily_date DESC
        LIMIT 1
        ",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(streak.flatten().unwrap_or(0))
}

pub async fn get_highest_streak(pool: &PgPool, user_id: Uuid) -> Result<i32, ApiError> {
    let streak = sqlx::query_scalar::<_, Option<i32>>(
        r"
        SELECT MAX(current_streak)
        FROM activities
        WHERE user_id = $1
            AND is_daily_flag = true
            AND is_solved = true
        ",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(streak.flatten().unwrap_or(0))
}

pub async fn get_average_attempts(pool: &PgPool, user_id: Uuid) -> Result<f64, ApiError> {
    let avg = sqlx::query_scalar::<_, Option<f64>>(
        r"
        SELECT AVG(attempts)
        FROM activities
        WHERE user_id = $1
            AND is_solved = true
        ",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(avg.flatten().unwrap_or(0.0))
}

pub async fn is_puzzle_solved(
    pool: &PgPool,
    user_id: Uuid,
    puzzle_id: Uuid,
) -> Result<bool, ApiError> {
    let is_solved = sqlx::query_scalar::<_, bool>(
        r"
        SELECT is_solved
        FROM activities
        WHERE user_id = $1
            AND puzzle_id = $2
        ",
    )
    .bind(user_id)
    .bind(puzzle_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(is_solved.unwrap_or(false))
}

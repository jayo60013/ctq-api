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

pub async fn get_puzzles_with_activities_by_date_range(
    pool: &PgPool,
    user_id: Uuid,
    from_date: NaiveDate,
    to_date: NaiveDate,
) -> Result<Vec<(NaiveDate, Uuid, Option<ActivityRow>)>, ApiError> {
    #[derive(sqlx::FromRow)]
    struct PuzzleWithOptionalActivity {
        daily_date: NaiveDate,
        puzzle_id: Uuid,
        user_id: Option<Uuid>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
        attempts: Option<i32>,
        checks_used: Option<i32>,
        solves_used: Option<i32>,
        is_solved: Option<bool>,
        is_daily_flag: Option<bool>,
        current_streak: Option<i32>,
    }

    let rows = sqlx::query_as::<_, PuzzleWithOptionalActivity>(
        r"
        SELECT p.daily_date, p.id as puzzle_id, a.user_id, a.completed_at, a.attempts, a.checks_used, a.solves_used, a.is_solved, a.is_daily_flag, a.current_streak
        FROM puzzles p
        LEFT JOIN activities a ON p.id = a.puzzle_id AND a.user_id = $1
        WHERE p.daily_date >= $2
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

    let results = rows
        .into_iter()
        .map(|row| {
            let activity = row.user_id.map(|user_id| ActivityRow {
                user_id,
                puzzle_id: row.puzzle_id,
                completed_at: row.completed_at,
                attempts: row.attempts.unwrap_or(0),
                checks_used: row.checks_used.unwrap_or(0),
                solves_used: row.solves_used.unwrap_or(0),
                is_solved: row.is_solved.unwrap_or(false),
                is_daily_flag: row.is_daily_flag.unwrap_or(false),
                current_streak: row.current_streak.unwrap_or(0),
            });
            (row.daily_date, row.puzzle_id, activity)
        })
        .collect();

    Ok(results)
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
    let avg = sqlx::query_scalar::<_, f64>(
        r"
        SELECT COALESCE(AVG(attempts)::FLOAT8, 0)
        FROM activities
        WHERE user_id = $1
            AND is_solved = true
        ",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(avg)
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

/// Increments `checks_used` or `solves_used` for a puzzle activity.
/// Creates a new activity record if it doesn't exist.
pub async fn increment_activity_usage(
    pool: &PgPool,
    user_id: Uuid,
    puzzle_id: Uuid,
    increment_checks: i32,
    increment_solves: i32,
) -> Result<(), ApiError> {
    let activity = get_activity(pool, user_id, puzzle_id).await?;

    let (new_checks, new_solves) = if let Some(existing) = activity {
        // Activity exists, increment
        (
            existing.checks_used + increment_checks,
            existing.solves_used + increment_solves,
        )
    } else {
        // New activity, create with given values
        (increment_checks, increment_solves)
    };

    sqlx::query(
        r"
        INSERT INTO activities (user_id, puzzle_id, attempts, checks_used, solves_used, is_solved, is_daily_flag, completed_at, current_streak)
        VALUES ($1, $2, 0, $3, $4, false, false, NULL, 0)
        ON CONFLICT (user_id, puzzle_id)
        DO UPDATE SET
            checks_used = $3,
            solves_used = $4
        ",
    )
    .bind(user_id)
    .bind(puzzle_id)
    .bind(new_checks)
    .bind(new_solves)
    .execute(pool)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(())
}

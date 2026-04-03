use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivityRow {
    pub user_id: Uuid,
    pub puzzle_id: i32,
    pub completed_at: Option<DateTime<Utc>>,
    pub attempts: i32,
    pub checks_used: i32,
    pub hints_used: i32,
    pub is_solved: bool,
    pub is_daily_flag: bool,
    pub current_streak: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRowDto {
    pub puzzle_id: i32,
    pub completed_at: Option<DateTime<Utc>>,
    pub attempts: i32,
    pub checks_used: i32,
    pub hints_used: i32,
    pub is_solved: bool,
    pub is_daily_flag: bool,
    pub current_streak: i32,
}

impl From<ActivityRow> for ActivityRowDto {
    fn from(row: ActivityRow) -> Self {
        ActivityRowDto {
            puzzle_id: row.puzzle_id,
            completed_at: row.completed_at,
            attempts: row.attempts,
            checks_used: row.checks_used,
            hints_used: row.hints_used,
            is_solved: row.is_solved,
            is_daily_flag: row.is_daily_flag,
            current_streak: row.current_streak,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ActivityUpdateRequest {
    pub checks_used: u16,
    pub solves_used: u16,
}

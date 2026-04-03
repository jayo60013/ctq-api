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
    pub solves_used: i32,
    pub is_solved: bool,
    pub is_daily_flag: bool,
    pub current_streak: i32,
}

#[derive(Debug, Deserialize)]
pub struct ActivityUpdateRequest {
    pub checks_used: u16,
    pub solves_used: u16,
}



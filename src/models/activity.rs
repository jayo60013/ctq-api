use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRow {
    pub user_id: Uuid,
    pub puzzle_id: Uuid,
    pub completed_at: Option<DateTime<Utc>>,
    pub attempts: i32,
    pub checks_used: i32,
    pub solves_used: i32,
    pub is_solved: bool,
    pub is_daily_flag: bool,
    pub current_streak: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivityState {
    pub completed_at: Option<DateTime<Utc>>,
    pub attempts: i32,
    pub checks_used: i32,
    pub solves_used: i32,
    pub is_solved: bool,
    pub is_daily_flag: bool,
    pub current_streak: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySummaryResponse {
    pub puzzle_id: Uuid,
    pub daily_date: NaiveDate,
    pub state: Option<ActivityState>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ActivityUpdateRequest {
    pub checks_used: u16,
    pub solves_used: u16,
}

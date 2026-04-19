use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    pub total_played_puzzles: i64,
    pub current_streak: i32,
    pub highest_streak: i32,
    pub average_attempts: f64,
}

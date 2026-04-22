use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScoreDistributionBucket {
    pub score: i32,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    pub total_played_puzzles: i64,
    pub current_streak: i32,
    pub highest_streak: i32,
    pub score_distribution: Vec<ScoreDistributionBucket>,
}

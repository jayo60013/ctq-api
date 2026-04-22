use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use validator::Validate;

use crate::models::{GlobalStats, ScoreDistributionBucket};

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckQuoteRequest {
    pub cipher_map: HashMap<char, char>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStats {
    pub current_streak: i32,
    pub best_streak: i32,
    pub average_score: f64,
    pub distribution: Vec<ScoreDistributionBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckQuoteState {
    pub player: PlayerStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global: Option<GlobalStats>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckQuoteResponse {
    pub is_quote_correct: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<CheckQuoteState>,
}

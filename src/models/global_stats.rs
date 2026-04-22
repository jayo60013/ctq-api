use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GlobalStatsBucket {
    pub score: String,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GlobalStats {
    pub average_score: f64,
    pub distribution: Vec<GlobalStatsBucket>,
    pub percentile: i32,
}

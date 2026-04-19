use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleState {
    pub solved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks_used: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solves_used: Option<i32>,
}

impl PuzzleState {
    pub fn not_solved() -> Self {
        PuzzleState {
            solved: false,
            quote: None,
            checks_used: None,
            solves_used: None,
        }
    }

    pub fn not_solved_with_usage(checks_used: i32, solves_used: i32) -> Self {
        PuzzleState {
            solved: false,
            quote: None,
            checks_used: Some(checks_used),
            solves_used: Some(solves_used),
        }
    }

    pub fn solved(quote: String, checks_used: i32, solves_used: i32) -> Self {
        PuzzleState {
            solved: true,
            quote: Some(quote),
            checks_used: Some(checks_used),
            solves_used: Some(solves_used),
        }
    }
}

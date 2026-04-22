use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::{GlobalStats, PlayerStats};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player: Option<PlayerStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global: Option<GlobalStats>,
}

impl PuzzleState {
    pub fn not_solved() -> Self {
        PuzzleState {
            solved: false,
            quote: None,
            checks_used: None,
            solves_used: None,
            player: None,
            global: None,
        }
    }

    pub fn not_solved_with_usage(checks_used: i32, solves_used: i32) -> Self {
        PuzzleState {
            solved: false,
            quote: None,
            checks_used: Some(checks_used),
            solves_used: Some(solves_used),
            player: None,
            global: None,
        }
    }

    #[allow(dead_code)]
    pub fn solved(quote: String, checks_used: i32, solves_used: i32) -> Self {
        PuzzleState {
            solved: true,
            quote: Some(quote),
            checks_used: Some(checks_used),
            solves_used: Some(solves_used),
            player: None,
            global: None,
        }
    }

    #[allow(dead_code)]
    pub fn solved_with_stats(
        quote: String,
        checks_used: i32,
        solves_used: i32,
        player: PlayerStats,
    ) -> Self {
        PuzzleState {
            solved: true,
            quote: Some(quote),
            checks_used: Some(checks_used),
            solves_used: Some(solves_used),
            player: Some(player),
            global: None,
        }
    }

    pub fn solved_with_stats_and_global(
        quote: String,
        checks_used: i32,
        solves_used: i32,
        player: PlayerStats,
        global: Option<GlobalStats>,
    ) -> Self {
        PuzzleState {
            solved: true,
            quote: Some(quote),
            checks_used: Some(checks_used),
            solves_used: Some(solves_used),
            player: Some(player),
            global,
        }
    }
}

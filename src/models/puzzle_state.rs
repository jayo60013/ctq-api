use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::{Game, GlobalStats, PlayerStats};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleState {
    pub solved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game: Option<Game>,
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
            game: None,
            player: None,
            global: None,
        }
    }

    pub fn not_solved_with_usage(checks_used: i32, solves_used: i32) -> Self {
        let score = 6 - (checks_used + (solves_used * 2));
        PuzzleState {
            solved: false,
            quote: None,
            game: Some(Game {
                score,
                checks_used,
                solves_used,
            }),
            player: None,
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
        let score = 6 - (checks_used + (solves_used * 2));
        PuzzleState {
            solved: true,
            quote: Some(quote),
            game: Some(Game {
                score,
                checks_used,
                solves_used,
            }),
            player: Some(player),
            global,
        }
    }
}

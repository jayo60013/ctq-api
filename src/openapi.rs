// Needed for OpenAPI macro
#![allow(clippy::needless_for_each)]

use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::models::{
    ActivityRow, ActivityState, ActivitySummaryResponse, AuthResponse, CheckLetterRequest,
    CheckLetterResponse, CheckQuoteRequest, CheckQuoteResponse, PuzzleResponse, PuzzleState,
    ScoreDistributionBucket, ScoreRange, SolveLetterRequest, SolveLetterResponse, StatsResponse,
};
use crate::routes::auth::google::AuthUrlResponse;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::health::health_check,
        crate::routes::puzzles::daily::get_daily_puzzle,
        crate::routes::puzzles::daily::check_daily_letter,
        crate::routes::puzzles::daily::solve_daily_letter,
        crate::routes::puzzles::daily::check_daily_quote,
        crate::routes::puzzles::archive::get_puzzle,
        crate::routes::puzzles::archive::check_letter,
        crate::routes::puzzles::archive::solve_letter,
        crate::routes::puzzles::archive::check_quote,
        crate::routes::auth::google::get_google_auth_url,
        crate::routes::auth::google::google_callback,
        crate::routes::auth::logout::logout,
        crate::routes::me::activities::get_activity_summary,
        crate::routes::me::stats::get_stats,
    ),
    components(
        schemas(
            PuzzleResponse,
            PuzzleState,
            CheckLetterRequest,
            CheckLetterResponse,
            CheckQuoteRequest,
            CheckQuoteResponse,
            SolveLetterRequest,
            SolveLetterResponse,
            AuthResponse,
            AuthUrlResponse,
            StatsResponse,
            ScoreDistributionBucket,
            ScoreRange,
            ActivityRow,
            ActivityState,
            ActivitySummaryResponse
        )
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Crack the Quote API",
        description = "API for the Crack the Quote puzzle game",
        version = "0.3.0",
        contact(
            name = "API Support"
        ),
    ),
    tags(
        (name = "Puzzles", description = "Puzzle endpoints"),
        (name = "Authentication", description = "Authentication endpoints"),
        (name = "Activities", description = "User activity endpoints"),
        (name = "User", description = "User profile endpoints"),
        (name = "Health", description = "Health check endpoint"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_token",
            SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
        );
    }
}

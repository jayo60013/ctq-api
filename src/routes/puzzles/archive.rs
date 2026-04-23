use actix_web::{get, post, web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use validator::Validate;

use crate::models::PuzzleResponse;
use crate::{
    error::ApiError,
    middleware,
    models::{
        CheckLetterRequest, CheckLetterResponse, CheckQuoteRequest, CheckQuoteResponse,
        PlayerStats, PuzzleState, SolveLetterRequest, SolveLetterResponse,
    },
    repository::{get_activity, increment_activity_usage, is_puzzle_solved, PuzzleRepository},
    services::{ActivityService, JwtService, PuzzleService},
    validators,
};

#[utoipa::path(
    get,
    path = "/puzzles/{id}",
    params(
        ("id" = uuid::Uuid, Path, description = "Puzzle ID")
    ),
    responses(
        (status = 200, description = "Puzzle retrieved successfully", body = PuzzleResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Puzzle not found"),
    ),
    tag = "Puzzles"
)]
#[get("/{id}")]
pub async fn get_puzzle(
    pool: web::Data<PgPool>,
    jwt_service: web::Data<JwtService>,
    id: web::Path<uuid::Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let user = middleware::extract_authenticated_user(&req, jwt_service.get_ref())?;

    let puzzle_id = *id;
    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(puzzle_id).await?;

    let is_solved = is_puzzle_solved(pool.get_ref(), user.id, puzzle_id)
        .await
        .unwrap_or(false);

    let state = if is_solved {
        // Get activity data
        if let Ok(Some(activity)) = get_activity(pool.get_ref(), user.id, puzzle_id).await {
            // Get global stats
            let score = 6 - (activity.checks_used + (activity.solves_used * 2));
            let global =
                ActivityService::build_global_stats(pool.get_ref(), puzzle_id, score).await?;

            PuzzleState::solved_with_stats_and_global(
                puzzle.quote.clone(),
                activity.checks_used,
                activity.solves_used,
                // Dummy player stats (not fetched for archive unless needed)
                PlayerStats {
                    current_streak: 0,
                    best_streak: 0,
                    average_score: 0.0,
                    total_puzzles_completed: 0,
                    distribution: vec![],
                },
                global,
            )
        } else {
            PuzzleState::not_solved()
        }
    } else {
        // Check if there's activity for this puzzle (checks/solves used)
        if let Ok(Some(activity)) = get_activity(pool.get_ref(), user.id, puzzle_id).await {
            PuzzleState::not_solved_with_usage(activity.checks_used, activity.solves_used)
        } else {
            PuzzleState::not_solved()
        }
    };

    let response = PuzzleResponse {
        id: puzzle.id,
        encoded_quote: puzzle.encoded_quote,
        author: puzzle.author,
        source: puzzle.source,
        date: puzzle.daily_date,
        state,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/puzzles/{id}/check-letter",
    params(
        ("id" = uuid::Uuid, Path, description = "Puzzle ID")
    ),
    request_body = CheckLetterRequest,
    responses(
        (status = 200, description = "Letter checked successfully", body = CheckLetterResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Puzzles"
)]
#[post("/{id}/check-letter")]
pub async fn check_letter(
    pool: web::Data<PgPool>,
    jwt_service: web::Data<JwtService>,
    id: web::Path<uuid::Uuid>,
    req: HttpRequest,
    body: web::Json<CheckLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    let user = middleware::extract_authenticated_user(&req, jwt_service.get_ref())?;

    body.validate()?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(*id).await?;

    let is_correct =
        PuzzleService::check_letter(body.cipher_letter, body.letter_to_check, &puzzle.cipher_map);

    // Track the check usage
    // Get current activity to check budget
    if let Ok(Some(activity)) = get_activity(pool.get_ref(), user.id, *id).await {
        validators::validate_budget(activity.checks_used, activity.solves_used, 1)?;
    }
    // Increment checks_used by 1
    increment_activity_usage(pool.get_ref(), user.id, *id, 1, 0).await?;

    let response = CheckLetterResponse {
        is_letter_correct: is_correct,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/puzzles/{id}/solve-letter",
    params(
        ("id" = uuid::Uuid, Path, description = "Puzzle ID")
    ),
    request_body = SolveLetterRequest,
    responses(
        (status = 200, description = "Letter solved successfully", body = SolveLetterResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Puzzles"
)]
#[post("/{id}/solve-letter")]
pub async fn solve_letter(
    pool: web::Data<PgPool>,
    jwt_service: web::Data<JwtService>,
    id: web::Path<uuid::Uuid>,
    req: HttpRequest,
    body: web::Json<SolveLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    let user = middleware::extract_authenticated_user(&req, jwt_service.get_ref())?;

    body.validate()?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(*id).await?;

    let correct_letter = PuzzleService::solve_letter(body.cipher_letter, &puzzle.cipher_map)?;

    // Track the solve usage
    // Get current activity to check budget
    if let Ok(Some(activity)) = get_activity(pool.get_ref(), user.id, *id).await {
        validators::validate_budget(activity.checks_used, activity.solves_used, 2)?;
    }
    // Increment solves_used by 1 (costs 2 points)
    increment_activity_usage(pool.get_ref(), user.id, *id, 0, 1).await?;

    let response = SolveLetterResponse { correct_letter };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/puzzles/{id}/check-quote",
    params(
        ("id" = uuid::Uuid, Path, description = "Puzzle ID")
    ),
    request_body = CheckQuoteRequest,
    responses(
        (status = 200, description = "Quote checked successfully", body = CheckQuoteResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Puzzles"
)]
#[post("/{id}/check-quote")]
pub async fn check_quote(
    pool: web::Data<PgPool>,
    jwt_service: web::Data<JwtService>,
    id: web::Path<uuid::Uuid>,
    req: HttpRequest,
    body: web::Json<CheckQuoteRequest>,
) -> Result<HttpResponse, ApiError> {
    let user = middleware::extract_authenticated_user(&req, jwt_service.get_ref())?;

    body.validate()?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(*id).await?;
    let is_correct = PuzzleService::check_quote(&body.cipher_map, &puzzle.cipher_map);

    if is_correct {
        let state = ActivityService::record_archive_solution(pool.get_ref(), user.id, *id).await?;

        let response = CheckQuoteResponse {
            is_quote_correct: true,
            state: Some(state),
        };
        Ok(HttpResponse::Ok().json(response))
    } else {
        let response = CheckQuoteResponse {
            is_quote_correct: false,
            state: None,
        };
        Ok(HttpResponse::Ok().json(response))
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_puzzle)
        .service(check_letter)
        .service(solve_letter)
        .service(check_quote);
}

use actix_web::{get, post, web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use validator::Validate;

use crate::models::PuzzleResponse;
use crate::services::ActivityService;
use crate::{
    config::EnvConfig,
    error::ApiError,
    middleware,
    models::{
        ActivityUpdateRequest, CheckLetterRequest, CheckLetterResponse, CheckQuoteRequest,
        CheckQuoteResponse, PuzzleState, SolveLetterRequest, SolveLetterResponse,
    },
    repository::{is_puzzle_solved, PuzzleRepository},
    services::{JwtService, PuzzleService},
    validators,
};

#[get("/{id}")]
async fn get_puzzle(
    pool: web::Data<PgPool>,
    config: web::Data<EnvConfig>,
    id: web::Path<uuid::Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let jwt_service = JwtService::new(&config.jwt_secret);
    let user = middleware::extract_authenticated_user(&req, &jwt_service)?;

    let puzzle_id = *id;
    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(puzzle_id).await?;

    let is_solved = is_puzzle_solved(pool.get_ref(), user.id, puzzle_id)
        .await
        .unwrap_or(false);

    let state = if is_solved {
        // Get activity data
        if let Ok(Some(activity)) =
            crate::repository::get_activity(pool.get_ref(), user.id, puzzle_id).await
        {
            PuzzleState::solved(
                puzzle.quote.clone(),
                activity.checks_used,
                activity.solves_used,
            )
        } else {
            PuzzleState::not_solved()
        }
    } else {
        PuzzleState::not_solved()
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

#[post("/{id}/check-letter")]
async fn check_letter(
    pool: web::Data<PgPool>,
    config: web::Data<EnvConfig>,
    id: web::Path<uuid::Uuid>,
    req: HttpRequest,
    body: web::Json<CheckLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    let jwt_service = JwtService::new(&config.jwt_secret);
    let _user = middleware::extract_authenticated_user(&req, &jwt_service)?;

    body.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(*id).await?;

    let is_correct =
        PuzzleService::check_letter(body.cipher_letter, body.letter_to_check, &puzzle.cipher_map);

    let response = CheckLetterResponse {
        is_letter_correct: is_correct,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[post("/{id}/solve-letter")]
async fn solve_letter(
    pool: web::Data<PgPool>,
    config: web::Data<EnvConfig>,
    id: web::Path<uuid::Uuid>,
    req: HttpRequest,
    body: web::Json<SolveLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    let jwt_service = JwtService::new(&config.jwt_secret);
    let _user = middleware::extract_authenticated_user(&req, &jwt_service)?;

    body.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(*id).await?;

    let correct_letter = PuzzleService::solve_letter(body.cipher_letter, &puzzle.cipher_map)?;

    let response = SolveLetterResponse { correct_letter };
    Ok(HttpResponse::Ok().json(response))
}

#[post("/{id}/check-quote")]
async fn check_quote(
    pool: web::Data<PgPool>,
    config: web::Data<EnvConfig>,
    id: web::Path<uuid::Uuid>,
    req: HttpRequest,
    body: web::Json<CheckQuoteRequest>,
) -> Result<HttpResponse, ApiError> {
    let jwt_service = JwtService::new(&config.jwt_secret);
    let user = middleware::extract_authenticated_user(&req, &jwt_service)?;

    body.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    validators::validate_activity_request(&ActivityUpdateRequest {
        checks_used: body.checks_used,
        solves_used: body.solves_used,
    })?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(*id).await?;

    let is_correct = PuzzleService::check_quote(&body.cipher_map, &puzzle.cipher_map);

    let activity_req = ActivityUpdateRequest {
        checks_used: body.checks_used,
        solves_used: body.solves_used,
    };
    let _ = ActivityService::track_activity(
        pool.get_ref(),
        user.id,
        *id,
        is_correct,
        false,
        &activity_req,
    )
    .await;

    let response = CheckQuoteResponse {
        is_quote_correct: is_correct,
    };
    Ok(HttpResponse::Ok().json(response))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_puzzle)
        .service(check_letter)
        .service(solve_letter)
        .service(check_quote);
}

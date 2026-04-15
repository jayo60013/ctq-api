use actix_web::{get, post, web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use validator::Validate;

use crate::services::ActivityService;
use crate::{
    config::EnvConfig,
    error::ApiError,
    middleware::extract_authenticated_user,
    models::{
        ActivityUpdateRequest, CheckLetterRequest, CheckLetterResponse, CheckQuoteRequest,
        CheckQuoteResponse, PuzzleResponse, SolveLetterRequest, SolveLetterResponse,
    },
    puzzle_cache::DailyPuzzleCache,
    repository::PuzzleRepository,
    services::{JwtService, PuzzleService},
    validators,
};

#[get("/daily")]
async fn get_daily_puzzle(
    pool: web::Data<PgPool>,
    cache: web::Data<DailyPuzzleCache>,
) -> Result<HttpResponse, ApiError> {
    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo).await?;

    let response = PuzzleResponse {
        id: puzzle.id,
        encoded_quote: puzzle.encoded_quote,
        author: puzzle.author,
        source: puzzle.source,
        date: puzzle.daily_date,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[post("/daily/check-letter")]
async fn check_daily_letter(
    pool: web::Data<PgPool>,
    cache: web::Data<DailyPuzzleCache>,
    req: web::Json<CheckLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo).await?;

    let is_correct =
        PuzzleService::check_letter(req.cipher_letter, req.letter_to_check, &puzzle.cipher_map);

    let response = CheckLetterResponse {
        is_letter_correct: is_correct,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[post("/daily/solve-letter")]
async fn solve_daily_letter(
    pool: web::Data<PgPool>,
    cache: web::Data<DailyPuzzleCache>,
    req: web::Json<SolveLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo).await?;

    let correct_letter = PuzzleService::solve_letter(req.cipher_letter, &puzzle.cipher_map)?;

    let response = SolveLetterResponse { correct_letter };
    Ok(HttpResponse::Ok().json(response))
}

#[post("/daily/check-quote")]
async fn check_daily_quote(
    pool: web::Data<PgPool>,
    config: web::Data<EnvConfig>,
    cache: web::Data<DailyPuzzleCache>,
    req: web::Json<CheckQuoteRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    validators::validate_activity_request(&ActivityUpdateRequest {
        checks_used: req.checks_used,
        solves_used: req.solves_used,
    })?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo).await?;

    let is_correct = PuzzleService::check_quote(&req.cipher_map, &puzzle.cipher_map);

    let mut streak = None;

    let jwt_service = JwtService::new(&config.jwt_secret);
    if let Ok(user) = extract_authenticated_user(&http_req, &jwt_service) {
        let activity_req = ActivityUpdateRequest {
            checks_used: req.checks_used,
            solves_used: req.solves_used,
        };
        let current_streak = ActivityService::track_activity(
            pool.get_ref(),
            user.id,
            puzzle.id,
            is_correct,
            true,
            &activity_req,
        )
        .await?;

        streak = Some(current_streak);
    }

    let response = CheckQuoteResponse {
        is_quote_correct: is_correct,
        streak,
    };
    Ok(HttpResponse::Ok().json(response))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_daily_puzzle)
        .service(check_daily_letter)
        .service(solve_daily_letter)
        .service(check_daily_quote);
}

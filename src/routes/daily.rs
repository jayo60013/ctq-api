use actix_web::{HttpResponse, get, post, web};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    config::Config,
    db::PuzzleRepository,
    error::ApiError,
    models::{
        CheckLetterRequest, CheckLetterResponse, CheckQuoteRequest, CheckQuoteResponse,
        SolveLetterRequest, SolveLetterResponse,
    },
    puzzle_cache::DailyPuzzleCache,
    services::PuzzleService,
    transformer::parse_cipher_map_from_json,
};

// GET /puzzles/daily
#[get("/daily")]
async fn get_daily_puzzle(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    cache: web::Data<DailyPuzzleCache>,
) -> Result<HttpResponse, ApiError> {
    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_response(&repo, &config).await?;

    Ok(HttpResponse::Ok().json(puzzle))
}

// POST /puzzles/daily/check-letter
#[post("/daily/check-letter")]
async fn check_daily_letter(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    cache: web::Data<DailyPuzzleCache>,
    req: web::Json<CheckLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo, &config).await?;
    let cipher_map = parse_cipher_map_from_json(&puzzle.cipher_map)?;

    let is_correct =
        PuzzleService::check_letter(req.cipher_letter, req.letter_to_check, &cipher_map);

    let response = CheckLetterResponse {
        is_letter_correct: is_correct,
    };
    Ok(HttpResponse::Ok().json(response))
}

// POST /puzzles/daily/solve-letter
#[post("/daily/solve-letter")]
async fn solve_daily_letter(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    cache: web::Data<DailyPuzzleCache>,
    req: web::Json<SolveLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo, &config).await?;
    let cipher_map = parse_cipher_map_from_json(&puzzle.cipher_map)?;

    let correct_letter = PuzzleService::solve_letter(req.cipher_letter, &cipher_map)?;

    let response = SolveLetterResponse { correct_letter };
    Ok(HttpResponse::Ok().json(response))
}

// POST /puzzles/daily/check-quote
#[post("/daily/check-quote")]
async fn check_daily_quote(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    cache: web::Data<DailyPuzzleCache>,
    req: web::Json<CheckQuoteRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo, &config).await?;
    let cipher_map = parse_cipher_map_from_json(&puzzle.cipher_map)?;

    let is_correct = PuzzleService::check_quote(&req.cipher_map, &cipher_map);

    let response = CheckQuoteResponse {
        is_quote_correct: is_correct,
    };
    Ok(HttpResponse::Ok().json(response))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_daily_puzzle)
        .service(check_daily_letter)
        .service(solve_daily_letter)
        .service(check_daily_quote);
}

use actix_web::{HttpResponse, get, post, web};
use chrono::Local;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    config::Config,
    db::PuzzleRepository,
    error::ApiError,
    models::{
        CheckLetterRequest, CheckLetterResponse, CheckQuoteRequest, CheckQuoteResponse,
        PuzzleResponse, SolveLetterRequest, SolveLetterResponse,
    },
    services::PuzzleService,
    transformer::parse_cipher_map_from_json,
};

// GET /puzzles/{id}
#[get("/{id}")]
async fn get_puzzle(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let puzzle_id = *id;

    // Check if puzzle is in the future
    let puzzle_date = config.start_date + chrono::Duration::days(i64::from(puzzle_id - 1));
    if puzzle_date > Local::now().date_naive() {
        return Err(ApiError::NotFound);
    }

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(puzzle_id).await?;

    let response = PuzzleResponse {
        id: puzzle.id,
        encoded_quote: puzzle.encoded_quote,
        author: puzzle.author,
        source: puzzle.source,
        date: puzzle.daily_date,
    };

    Ok(HttpResponse::Ok().json(response))
}

// POST /puzzles/{id}/check-letter
#[post("/{id}/check-letter")]
async fn check_letter(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    req: web::Json<CheckLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(*id).await?;
    let cipher_map = parse_cipher_map_from_json(&puzzle.cipher_map)?;

    let is_correct =
        PuzzleService::check_letter(req.cipher_letter, req.letter_to_check, &cipher_map);

    let response = CheckLetterResponse {
        is_letter_correct: is_correct,
    };
    Ok(HttpResponse::Ok().json(response))
}

// POST /puzzles/{id}/solve-letter
#[post("/{id}/solve-letter")]
async fn solve_letter(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    req: web::Json<SolveLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(*id).await?;
    let cipher_map = parse_cipher_map_from_json(&puzzle.cipher_map)?;

    let correct_letter = PuzzleService::solve_letter(req.cipher_letter, &cipher_map)?;

    let response = SolveLetterResponse { correct_letter };
    Ok(HttpResponse::Ok().json(response))
}

// POST /puzzles/{id}/check-quote
#[post("/{id}/check-quote")]
async fn check_quote(
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    req: web::Json<CheckQuoteRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(*id).await?;
    let cipher_map = parse_cipher_map_from_json(&puzzle.cipher_map)?;

    let is_correct = PuzzleService::check_quote(&req.cipher_map, &cipher_map);

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

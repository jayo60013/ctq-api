use actix_web::{get, post, web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use validator::Validate;

use crate::models::PuzzleResponse;
use crate::{
    config::EnvConfig,
    error::ApiError,
    middleware::extract_authenticated_user,
    models::{
        CheckLetterRequest, CheckLetterResponse, CheckQuoteRequest, CheckQuoteResponse,
        SolveLetterRequest, SolveLetterResponse,
    },
    repository::PuzzleRepository,
    services::{JwtService, PuzzleService},
};

#[get("/{id}")]
async fn get_puzzle(
    pool: web::Data<PgPool>,
    config: web::Data<EnvConfig>,
    id: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let jwt_service = JwtService::new(&config.jwt_secret);
    let _user = extract_authenticated_user(&req, &jwt_service)?;

    let puzzle_id = *id;
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

#[post("/{id}/check-letter")]
async fn check_letter(
    pool: web::Data<PgPool>,
    config: web::Data<EnvConfig>,
    id: web::Path<i32>,
    req: HttpRequest,
    body: web::Json<CheckLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    let jwt_service = JwtService::new(&config.jwt_secret);
    let _user = extract_authenticated_user(&req, &jwt_service)?;

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
    id: web::Path<i32>,
    req: HttpRequest,
    body: web::Json<SolveLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    let jwt_service = JwtService::new(&config.jwt_secret);
    let _user = extract_authenticated_user(&req, &jwt_service)?;

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
    id: web::Path<i32>,
    req: HttpRequest,
    body: web::Json<CheckQuoteRequest>,
) -> Result<HttpResponse, ApiError> {
    let jwt_service = JwtService::new(&config.jwt_secret);
    let _user = extract_authenticated_user(&req, &jwt_service)?;

    body.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = repo.get_by_id(*id).await?;

    let is_correct = PuzzleService::check_quote(&body.cipher_map, &puzzle.cipher_map);

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

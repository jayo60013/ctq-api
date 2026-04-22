use actix_web::{get, post, web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    error::ApiError,
    middleware::extract_authenticated_user,
    models::PuzzleState,
    models::{
        CheckLetterRequest, CheckLetterResponse, CheckQuoteRequest, CheckQuoteResponse,
        PuzzleResponse, SolveLetterRequest, SolveLetterResponse,
    },
    puzzle_cache::DailyPuzzleCache,
    repository::{get_activity, increment_activity_usage, is_puzzle_solved, PuzzleRepository},
    services::{ActivityService, JwtService, PuzzleService},
    validators,
};

#[utoipa::path(
    get,
    path = "/puzzles/daily",
    responses(
        (status = 200, description = "Daily puzzle retrieved successfully", body = PuzzleResponse),
        (status = 503, description = "Puzzle not generated yet"),
    ),
    tag = "Puzzles"
)]
#[get("/daily")]
pub async fn get_daily_puzzle(
    pool: web::Data<PgPool>,
    cache: web::Data<DailyPuzzleCache>,
    jwt_service: web::Data<JwtService>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo).await?;

    let user_opt = extract_authenticated_user(&req, jwt_service.get_ref()).ok();

    let state;

    if let Some(user) = user_opt {
        let is_solved = is_puzzle_solved(pool.get_ref(), user.id, puzzle.id)
            .await
            .unwrap_or(false);

        if is_solved {
            // Get activity data
            if let Ok(Some(activity)) = get_activity(pool.get_ref(), user.id, puzzle.id).await {
                // Fetch player stats for solved puzzle
                let player_stats =
                    ActivityService::build_player_stats(pool.get_ref(), user.id).await?;

                // Get global stats
                let score = activity.checks_used + (activity.solves_used * 2);
                let global =
                    ActivityService::build_global_stats(pool.get_ref(), puzzle.id, score).await?;

                state = PuzzleState::solved_with_stats_and_global(
                    puzzle.quote.clone(),
                    activity.checks_used,
                    activity.solves_used,
                    player_stats,
                    global,
                );
            } else {
                state = PuzzleState::not_solved();
            }
        } else {
            // Check if there's activity for this puzzle (checks/solves used)
            if let Ok(Some(activity)) = get_activity(pool.get_ref(), user.id, puzzle.id).await {
                state =
                    PuzzleState::not_solved_with_usage(activity.checks_used, activity.solves_used);
            } else {
                state = PuzzleState::not_solved();
            }
        }
    } else {
        state = PuzzleState::not_solved();
    }

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
    path = "/puzzles/daily/check-letter",
    request_body = CheckLetterRequest,
    responses(
        (status = 200, description = "Letter checked successfully", body = CheckLetterResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Puzzles"
)]
#[post("/daily/check-letter")]
pub async fn check_daily_letter(
    pool: web::Data<PgPool>,
    cache: web::Data<DailyPuzzleCache>,
    jwt_service: web::Data<JwtService>,
    req: HttpRequest,
    body: web::Json<CheckLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    body.validate()?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo).await?;

    let is_correct =
        PuzzleService::check_letter(body.cipher_letter, body.letter_to_check, &puzzle.cipher_map);

    // If user is authenticated, track the check usage
    if let Ok(user) = extract_authenticated_user(&req, jwt_service.get_ref()) {
        // Get current activity to check budget
        if let Ok(Some(activity)) = get_activity(pool.get_ref(), user.id, puzzle.id).await {
            validators::validate_budget(activity.checks_used, activity.solves_used, 1)?;
        }
        // Increment checks_used by 1
        increment_activity_usage(pool.get_ref(), user.id, puzzle.id, 1, 0).await?;
    }

    let response = CheckLetterResponse {
        is_letter_correct: is_correct,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/puzzles/daily/solve-letter",
    request_body = SolveLetterRequest,
    responses(
        (status = 200, description = "Letter solved successfully", body = SolveLetterResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Puzzles"
)]
#[post("/daily/solve-letter")]
pub async fn solve_daily_letter(
    pool: web::Data<PgPool>,
    cache: web::Data<DailyPuzzleCache>,
    jwt_service: web::Data<JwtService>,
    req: HttpRequest,
    body: web::Json<SolveLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    body.validate()?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo).await?;

    let correct_letter = PuzzleService::solve_letter(body.cipher_letter, &puzzle.cipher_map)?;

    // If user is authenticated, track the solve usage
    if let Ok(user) = extract_authenticated_user(&req, jwt_service.get_ref()) {
        // Get current activity to check budget
        if let Ok(Some(activity)) = get_activity(pool.get_ref(), user.id, puzzle.id).await {
            validators::validate_budget(activity.checks_used, activity.solves_used, 2)?;
        }
        // Increment solves_used by 1 (costs 2 points)
        increment_activity_usage(pool.get_ref(), user.id, puzzle.id, 0, 1).await?;
    }

    let response = SolveLetterResponse { correct_letter };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/puzzles/daily/check-quote",
    request_body = CheckQuoteRequest,
    responses(
        (status = 200, description = "Quote checked successfully", body = CheckQuoteResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Puzzles"
)]
#[post("/daily/check-quote")]
pub async fn check_daily_quote(
    pool: web::Data<PgPool>,
    cache: web::Data<DailyPuzzleCache>,
    jwt_service: web::Data<JwtService>,
    req: HttpRequest,
    body: web::Json<CheckQuoteRequest>,
    repo: web::Data<PuzzleRepository>,
) -> Result<HttpResponse, ApiError> {
    body.validate()?;

    let puzzle = cache.get_puzzle(repo.get_ref()).await?;
    let is_correct = PuzzleService::check_quote(&body.cipher_map, &puzzle.cipher_map);

    // If user is authenticated and quote is correct, handle stats and mark as solved
    if is_correct {
        if let Ok(user) = extract_authenticated_user(&req, jwt_service.get_ref()) {
            let (score, state) =
                ActivityService::record_solution(pool.get_ref(), user.id, puzzle.id).await?;

            let response = CheckQuoteResponse {
                is_quote_correct: true,
                score: Some(score),
                state: Some(state),
            };
            Ok(HttpResponse::Ok().json(response))
        } else {
            // Correct but unauthenticated
            let response = CheckQuoteResponse {
                is_quote_correct: true,
                score: None,
                state: None,
            };
            Ok(HttpResponse::Ok().json(response))
        }
    } else {
        // Incorrect answer
        let response = CheckQuoteResponse {
            is_quote_correct: false,
            score: None,
            state: None,
        };
        Ok(HttpResponse::Ok().json(response))
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_daily_puzzle)
        .service(check_daily_letter)
        .service(solve_daily_letter)
        .service(check_daily_quote);
}

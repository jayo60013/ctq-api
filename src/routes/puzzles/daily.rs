use actix_web::{get, post, web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    config::EnvConfig,
    error::ApiError,
    middleware::extract_authenticated_user,
    models::{
        CheckLetterRequest, CheckLetterResponse, CheckQuoteRequest, CheckQuoteResponse,
        PlayerStats, PuzzleResponse, ScoreDistributionBucket, ScoreRange, SolveLetterRequest,
        SolveLetterResponse,
    },
    models::{CheckQuoteState, GlobalStats, GlobalStatsBucket, PuzzleState},
    puzzle_cache::DailyPuzzleCache,
    repository::{
        get_activity, get_assist_budget_distribution, get_average_score, get_current_streak,
        get_highest_streak, get_puzzle_global_stats, get_puzzle_percentile,
        get_total_solved_puzzles, increment_activity_usage, is_puzzle_solved,
        update_puzzle_global_stats, upsert_activity, PuzzleRepository,
    },
    services::{JwtService, PuzzleService},
    validators,
};

/// Helper function to build `GlobalStats` from puzzle global stats data
async fn build_global_stats(
    pool: &PgPool,
    puzzle_id: uuid::Uuid,
    user_score: i32,
) -> Result<Option<GlobalStats>, ApiError> {
    if let Some((solve_count, total_score_sum, score_distribution)) =
        get_puzzle_global_stats(pool, puzzle_id).await?
    {
        let average_score = if solve_count > 0 {
            #[allow(clippy::cast_precision_loss)]
            {
                total_score_sum as f64 / solve_count as f64
            }
        } else {
            0.0
        };

        let distribution = score_distribution
            .iter()
            .enumerate()
            .map(|(idx, count)| {
                let percentage = if solve_count > 0 {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        (*count as f64 / solve_count as f64) * 100.0
                    }
                } else {
                    0.0
                };
                GlobalStatsBucket {
                    score: idx.to_string(),
                    percentage,
                }
            })
            .collect();

        let percentile = get_puzzle_percentile(pool, puzzle_id, user_score).await?;

        Ok(Some(GlobalStats {
            average_score,
            distribution,
            percentile,
        }))
    } else {
        Ok(None)
    }
}

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
    config: web::Data<EnvConfig>,
    cache: web::Data<DailyPuzzleCache>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo).await?;

    let user_opt = extract_authenticated_user(&req, &JwtService::new(&config.jwt_secret)).ok();

    let state;

    if let Some(user) = user_opt {
        let is_solved = is_puzzle_solved(pool.get_ref(), user.id, puzzle.id)
            .await
            .unwrap_or(false);

        if is_solved {
            // Get activity data
            if let Ok(Some(activity)) = get_activity(pool.get_ref(), user.id, puzzle.id).await {
                // Fetch player stats for solved puzzle
                let current_streak = get_current_streak(pool.get_ref(), user.id).await?;
                let best_streak = get_highest_streak(pool.get_ref(), user.id).await?;
                let average_score = get_average_score(pool.get_ref(), user.id).await?;
                let distribution_data =
                    get_assist_budget_distribution(pool.get_ref(), user.id).await?;
                let total_solved = get_total_solved_puzzles(pool.get_ref(), user.id).await?;

                // Calculate distribution with percentages
                let distribution = distribution_data
                    .iter()
                    .map(|(min, max, count)| ScoreDistributionBucket {
                        range: ScoreRange {
                            min: *min,
                            max: *max,
                        },
                        count: *count,
                        percentage: if total_solved > 0 {
                            #[allow(clippy::cast_precision_loss)]
                            {
                                *count as f64 / total_solved as f64
                            }
                        } else {
                            0.0
                        },
                    })
                    .collect();

                let player_stats = PlayerStats {
                    current_streak,
                    best_streak,
                    average_score,
                    distribution,
                };

                // Get global stats
                let score = activity.checks_used + (activity.solves_used * 2);
                let global = build_global_stats(pool.get_ref(), puzzle.id, score).await?;

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
    config: web::Data<EnvConfig>,
    cache: web::Data<DailyPuzzleCache>,
    req: HttpRequest,
    body: web::Json<CheckLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    body.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo).await?;

    let is_correct =
        PuzzleService::check_letter(body.cipher_letter, body.letter_to_check, &puzzle.cipher_map);

    // If user is authenticated, track the check usage
    let jwt_service = JwtService::new(&config.jwt_secret);
    if let Ok(user) = extract_authenticated_user(&req, &jwt_service) {
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
    config: web::Data<EnvConfig>,
    cache: web::Data<DailyPuzzleCache>,
    req: HttpRequest,
    body: web::Json<SolveLetterRequest>,
) -> Result<HttpResponse, ApiError> {
    body.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let repo = PuzzleRepository::new(pool.get_ref().clone());
    let puzzle = cache.get_puzzle(&repo).await?;

    let correct_letter = PuzzleService::solve_letter(body.cipher_letter, &puzzle.cipher_map)?;

    // If user is authenticated, track the solve usage
    let jwt_service = JwtService::new(&config.jwt_secret);
    if let Ok(user) = extract_authenticated_user(&req, &jwt_service) {
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
    config: web::Data<EnvConfig>,
    cache: web::Data<DailyPuzzleCache>,
    req: HttpRequest,
    body: web::Json<CheckQuoteRequest>,
    repo: web::Data<PuzzleRepository>,
) -> Result<HttpResponse, ApiError> {
    body.validate()
        .map_err(|e| ApiError::ValidationError(format!("{e:?}")))?;

    let puzzle = cache.get_puzzle(repo.get_ref()).await?;

    let is_correct = PuzzleService::check_quote(&body.cipher_map, &puzzle.cipher_map);

    // If user is authenticated and quote is correct, handle stats and mark as solved
    if is_correct {
        if let Ok(user) = extract_authenticated_user(&req, &JwtService::new(&config.jwt_secret)) {
            // Check if this was previously solved
            let was_already_solved = is_puzzle_solved(pool.get_ref(), user.id, puzzle.id)
                .await
                .unwrap_or(false);

            // Get current activity to get current stats
            let activity = get_activity(pool.get_ref(), user.id, puzzle.id).await?;
            let (checks_used, solves_used) = if let Some(a) = activity {
                (a.checks_used, a.solves_used)
            } else {
                (0, 0)
            };

            // Mark puzzle as solved
            upsert_activity(
                pool.get_ref(),
                user.id,
                puzzle.id,
                checks_used,
                solves_used,
                true,
                true,
            )
            .await?;

            // Calculate score
            let score = checks_used + (solves_used * 2);

            // Update global stats only on first-time completion
            if !was_already_solved {
                update_puzzle_global_stats(pool.get_ref(), puzzle.id, score).await?;
            }

            // Get player stats
            let current_streak = get_current_streak(pool.get_ref(), user.id).await?;
            let best_streak = get_highest_streak(pool.get_ref(), user.id).await?;
            let average_score = get_average_score(pool.get_ref(), user.id).await?;
            let distribution_data = get_assist_budget_distribution(pool.get_ref(), user.id).await?;
            let total_solved = get_total_solved_puzzles(pool.get_ref(), user.id).await?;

            // Calculate distribution with percentages
            let distribution = distribution_data
                .iter()
                .map(|(min, max, count)| ScoreDistributionBucket {
                    range: ScoreRange {
                        min: *min,
                        max: *max,
                    },
                    count: *count,
                    percentage: if total_solved > 0 {
                        #[allow(clippy::cast_precision_loss)]
                        {
                            *count as f64 / total_solved as f64
                        }
                    } else {
                        0.0
                    },
                })
                .collect();

            // Get global stats
            let global = build_global_stats(pool.get_ref(), puzzle.id, score).await?;

            let response = CheckQuoteResponse {
                is_quote_correct: true,
                score: Some(score),
                state: Some(CheckQuoteState {
                    player: PlayerStats {
                        current_streak,
                        best_streak,
                        average_score,
                        distribution,
                    },
                    global,
                }),
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

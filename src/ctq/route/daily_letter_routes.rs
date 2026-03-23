use crate::{
    DailyPuzzleCache,
    ctq::model::letter_payloads::{LetterCheckPayload, LetterSolvePayload},
};
use actix_web::{HttpResponse, Responder, post, web};
use serde_json::json;
use validator::Validate;

// /daily/letter/check
#[post("/check")]
pub async fn check_letter(
    cache: web::Data<DailyPuzzleCache>,
    payload: web::Json<LetterCheckPayload>,
) -> impl Responder {
    if let Err(e) = payload.validate() {
        return HttpResponse::BadRequest().json(e);
    }

    let daily_puzzle = &*cache.read().await;

    let is_correct = daily_puzzle
        .cipher_map
        .get(&payload.cipher_letter)
        .map(|&correct_letter| correct_letter == payload.letter_to_check)
        .unwrap_or(false);
    HttpResponse::Ok().json(json!({
        "isLetterCorrect": is_correct
    }))
}

// /daily/letter/solve
#[post("/solve")]
pub async fn solve_letter(
    cache: web::Data<DailyPuzzleCache>,
    payload: web::Json<LetterSolvePayload>,
) -> impl Responder {
    if let Err(e) = payload.validate() {
        return HttpResponse::BadRequest().json(e);
    }

    let daily_puzzle = &*cache.read().await;

    daily_puzzle
        .cipher_map
        .get(&payload.cipher_letter)
        .map(|&correct_letter| HttpResponse::Ok().json(json!({ "correctLetter": correct_letter})))
        .unwrap_or_else(|| {
            HttpResponse::BadRequest().json(json!({"error": "Letter not in today's puzzle"}))
        })
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(check_letter).service(solve_letter);
}

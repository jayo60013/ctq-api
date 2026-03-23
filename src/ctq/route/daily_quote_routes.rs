use crate::{DailyPuzzleCache, ctq::model::quote_payloads::QuoteCheckPayload};
use actix_web::{HttpResponse, Responder, get, post, web};
use serde_json::json;
use validator::Validate;

// /daily/quote/check
#[post("/check")]
pub async fn check_quote(
    cache: web::Data<DailyPuzzleCache>,
    payload: web::Json<QuoteCheckPayload>,
) -> impl Responder {
    if let Err(e) = payload.validate() {
        return HttpResponse::BadRequest().json(e);
    }

    let daily_puzzle = &*cache.read().await;
    let is_quote_correct = payload.cipher_map == daily_puzzle.cipher_map;

    HttpResponse::Ok().json(json!({
        "isQuoteCorrect": is_quote_correct
    }))
}

// /daily/quote/solve
#[get("/solve")]
pub async fn solve_quote(_cache: web::Data<DailyPuzzleCache>) -> impl Responder {
    HttpResponse::NotImplemented()
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(check_quote).service(solve_quote);
}

use actix_web::{HttpResponse, Responder, get, web};

use crate::DailyPuzzleResponseCache;

// /daily
#[get("")]
pub async fn daily_puzzle(cache: web::Data<DailyPuzzleResponseCache>) -> impl Responder {
    let daily_puzzle_response = &*cache.read().await;

    HttpResponse::Ok().json(serde_json::json!(daily_puzzle_response))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(daily_puzzle);
}

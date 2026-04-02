use actix_web::{HttpResponse, get};

#[get("/health")]
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(health_check);
}

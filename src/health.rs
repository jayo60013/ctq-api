use actix_web::{get, HttpResponse};

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy"),
    ),
    tag = "Health"
)]
#[get("/health")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(health_check);
}

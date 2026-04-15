use actix_web::{get, web, HttpRequest, HttpResponse};
use sqlx::PgPool;

use crate::config::EnvConfig;
use crate::error::ApiError;
use crate::middleware::extract_authenticated_user;
use crate::services::{ActivityService, JwtService};

#[get("/stats")]
async fn get_stats(
    pool: web::Data<PgPool>,
    config: web::Data<EnvConfig>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let jwt_service = JwtService::new(&config.jwt_secret);
    let user = extract_authenticated_user(&req, &jwt_service)?;

    let stats = ActivityService::get_stats(pool.get_ref(), user.id).await?;

    Ok(HttpResponse::Ok().json(stats))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_stats);
}

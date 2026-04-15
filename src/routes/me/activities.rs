use actix_web::{get, web, HttpRequest, HttpResponse};
use sqlx::PgPool;

use crate::config::EnvConfig;
use crate::error::ApiError;
use crate::middleware::extract_authenticated_user;
use crate::services::{ActivityService, JwtService};
use crate::validators::DateRange;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    pub from: String,
    pub to: String,
}

#[get("/summary")]
async fn get_activity_summary(
    pool: web::Data<PgPool>,
    config: web::Data<EnvConfig>,
    req: HttpRequest,
    query: web::Query<QueryParams>,
) -> Result<HttpResponse, ApiError> {
    let jwt_service = JwtService::new(&config.jwt_secret);
    let user = extract_authenticated_user(&req, &jwt_service)?;

    let date_range = DateRange::new(&query.from, &query.to)?;

    let activities = ActivityService::fetch_activity_summary(
        pool.get_ref(),
        user.id,
        date_range.from,
        date_range.to,
    )
    .await?;

    Ok(HttpResponse::Ok().json(activities))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_activity_summary);
}

use actix_web::{get, web, HttpRequest, HttpResponse};
use sqlx::PgPool;

use crate::config::EnvConfig;
use crate::error::ApiError;
use crate::middleware::extract_authenticated_user;
use crate::models::ActivitySummaryResponse;
use crate::services::{ActivityService, JwtService};
use crate::validators::DateRange;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    pub from: String,
    pub to: String,
}

#[utoipa::path(
    get,
    path = "/me/activities/summary",
    params(
        ("from" = String, Query, description = "Start date (YYYY-MM-DD)"),
        ("to" = String, Query, description = "End date (YYYY-MM-DD)"),
    ),
    responses(
        (status = 200, description = "Activity summary retrieved", body = Vec<ActivitySummaryResponse>),
        (status = 400, description = "Invalid date range"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Activities"
)]
#[get("/summary")]
pub async fn get_activity_summary(
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

use actix_web::{get, web, HttpRequest, HttpResponse};
use sqlx::PgPool;

use crate::error::ApiError;
use crate::middleware::extract_authenticated_user;
use crate::models::PlayerStats;
use crate::services::{ActivityService, JwtService};

#[utoipa::path(
    get,
    path = "/me/stats",
    responses(
        (status = 200, description = "User statistics retrieved", body = PlayerStats),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "User"
)]
#[get("/stats")]
pub async fn get_stats(
    pool: web::Data<PgPool>,
    jwt_service: web::Data<JwtService>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let user = extract_authenticated_user(&req, jwt_service.get_ref())?;

    let player_stats = ActivityService::build_player_stats(pool.get_ref(), user.id).await?;

    Ok(HttpResponse::Ok().json(player_stats))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_stats);
}

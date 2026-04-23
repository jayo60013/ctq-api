use actix_web::{get, web, HttpRequest, HttpResponse};
use sqlx::PgPool;

use crate::{
    error::ApiError, middleware::auth_middleware::extract_authenticated_user, models::AuthResponse,
    repository::UserRepository, services::JwtService,
};

/// GET /auth/me
/// Validates the current auth token and returns user information
/// Returns 200 with user details if token is valid, 401 if expired/invalid
#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Token is valid, user information returned", body = AuthResponse),
        (status = 401, description = "Token is invalid or expired"),
    ),
    tag = "Authentication"
)]
#[get("/me")]
pub async fn me(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    jwt_service: web::Data<JwtService>,
) -> Result<HttpResponse, ApiError> {
    // Extract authenticated user from token - returns 401 if invalid/expired
    let authenticated_user = extract_authenticated_user(&req, &jwt_service)?;

    // Fetch full user information from database
    let user_repo = UserRepository::new(pool.get_ref().clone());
    let user = user_repo.get_by_id(authenticated_user.id).await?;

    let response = AuthResponse {
        user_id: user.id.to_string(),
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(me);
}

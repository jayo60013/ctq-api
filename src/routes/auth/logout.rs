use actix_web::{cookie::Cookie, post, web, HttpResponse};

use crate::config::EnvConfig;
use crate::error::ApiError;

// POST /auth/logout
#[post("/logout")]
async fn logout(config: web::Data<EnvConfig>) -> Result<HttpResponse, ApiError> {
    let mut cookie = Cookie::new("auth_token", "");
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_secure(Some(config.secure_cookies));
    cookie.set_same_site(actix_web::cookie::SameSite::Lax);
    cookie.set_max_age(actix_web::cookie::time::Duration::seconds(0));

    let mut response = HttpResponse::Ok().json(serde_json::json!({"message": "Logged out"}));
    response
        .add_cookie(&cookie)
        .map_err(|_| ApiError::ExternalServiceError("Failed to clear cookie".to_string()))?;

    Ok(response)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(logout);
}

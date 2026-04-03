use actix_web::{cookie::Cookie, post, HttpResponse};

use crate::error::ApiError;

#[post("/logout")]
async fn logout() -> Result<HttpResponse, ApiError> {
    let mut cookie = Cookie::new("auth_token", "");
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_max_age(actix_web::cookie::time::Duration::seconds(0));

    let mut response = HttpResponse::Ok().finish();
    response
        .add_cookie(&cookie)
        .map_err(|_| ApiError::ExternalServiceError("Failed to clear cookie".to_string()))?;

    Ok(response)
}

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(logout);
}

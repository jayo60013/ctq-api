use actix_web::{cookie::Cookie, get, web, HttpResponse};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    config::EnvConfig,
    error::ApiError,
    models::AuthResponse,
    repository::UserRepository,
    services::{GoogleOAuthService, JwtService},
};

#[derive(Serialize, ToSchema)]
pub struct AuthUrlResponse {
    pub url: String,
    pub state: String,
    pub code_verifier: String,
}

// GET /auth/google/url
#[utoipa::path(
    get,
    path = "/auth/google/url",
    responses(
        (status = 200, description = "Google authentication URL generated", body = AuthUrlResponse),
    ),
    tag = "Authentication"
)]
#[get("/google/url")]
pub async fn get_google_auth_url(config: web::Data<EnvConfig>) -> Result<HttpResponse, ApiError> {
    let (code_verifier, code_challenge) = GoogleOAuthService::generate_pkce_pair();
    let state = Uuid::new_v4().to_string();

    let oauth_service = GoogleOAuthService::new(
        config.google_client_id.clone(),
        config.google_client_secret.clone(),
        config.google_redirect_uri.clone(),
    );

    let auth_url = oauth_service.create_auth_url(&state, &code_challenge);

    let response = AuthUrlResponse {
        url: auth_url,
        state,
        code_verifier,
    };

    Ok(HttpResponse::Ok().json(response))
}

// GET /auth/google/callback
#[utoipa::path(
    get,
    path = "/auth/google/callback",
    params(
        ("code" = String, Query, description = "Authorization code from Google"),
        ("state" = String, Query, description = "State parameter for CSRF protection"),
        ("code_verifier" = String, Query, description = "PKCE code verifier"),
    ),
    responses(
        (status = 200, description = "Authentication successful", body = AuthResponse),
        (status = 400, description = "Invalid parameters"),
        (status = 401, description = "Authentication failed"),
    ),
    tag = "Authentication"
)]
#[get("/google/callback")]
pub async fn google_callback(
    pool: web::Data<PgPool>,
    config: web::Data<EnvConfig>,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse, ApiError> {
    let code = query
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::ValidationError("Missing code parameter".to_string()))?;

    let state = query
        .get("state")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::ValidationError("Missing state parameter".to_string()))?;

    let code_verifier = query
        .get("code_verifier")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::ValidationError("Missing code_verifier in query".to_string()))?;

    if state.is_empty() || code_verifier.is_empty() {
        return Err(ApiError::ValidationError(
            "State and code_verifier must not be empty".to_string(),
        ));
    }

    let oauth_service = GoogleOAuthService::new(
        config.google_client_id.clone(),
        config.google_client_secret.clone(),
        config.google_redirect_uri.clone(),
    );

    let id_token = oauth_service
        .exchange_code_for_token(code, code_verifier)
        .await?;

    let google_payload = oauth_service
        .verify_id_token(&id_token.0, &id_token.1)
        .await?;

    let user_repo = UserRepository::new(pool.get_ref().clone());
    let user = user_repo
        .create_or_update(
            &google_payload.user_id,
            &google_payload.email,
            google_payload.name.as_deref(),
            google_payload.picture.as_deref(),
        )
        .await?;

    let jwt_service = JwtService::new(&config.jwt_secret);
    let token = jwt_service.create_token(&user, 24)?;

    let response = AuthResponse {
        user_id: user.id.to_string(),
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
    };

    let mut http_response = HttpResponse::Ok().json(response);

    let mut cookie = Cookie::new("auth_token", token);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_secure(Some(config.secure_cookies));
    cookie.set_same_site(actix_web::cookie::SameSite::Lax);

    http_response
        .add_cookie(&cookie)
        .map_err(|_| ApiError::ExternalServiceError("Failed to set cookie".to_string()))?;

    Ok(http_response)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_google_auth_url).service(google_callback);
}

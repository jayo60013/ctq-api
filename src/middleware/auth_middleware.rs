use actix_web::HttpRequest;

use crate::error::ApiError;
use crate::models::AuthenticatedUser;
use crate::services::JwtService;

/// Extracts and validates authenticated user from `HttpOnly` cookie.
///
/// This is the primary authentication mechanism for the API.
/// The `auth_token` cookie contains a JWT with user identity.
///
/// # Process
/// 1. Reads `auth_token` from `HttpRequest` cookies
/// 2. Verifies JWT signature and expiry
/// 3. Extracts user ID and email from JWT claims
/// 4. Returns `AuthenticatedUser` principal
///
/// # Returns
/// - `Ok(AuthenticatedUser)` if cookie is valid and JWT verifies
/// - `Err(ApiError::Unauthorized)` if cookie is missing or invalid
/// - `Err(ApiError::JwtError)` if JWT verification fails
pub fn extract_authenticated_user(
    req: &HttpRequest,
    jwt_service: &JwtService,
) -> Result<AuthenticatedUser, ApiError> {
    let token = extract_jwt_from_cookies(req)?;
    let payload = jwt_service.verify_token(&token)?;

    let user_id = uuid::Uuid::parse_str(&payload.sub)
        .map_err(|_| ApiError::JwtError("Invalid user ID in token".to_string()))?;

    Ok(AuthenticatedUser {
        id: user_id,
        email: payload.email,
    })
}

/// Reads JWT from `auth_token` cookie.
///
/// Returns Unauthorized error if cookie is missing or empty.
fn extract_jwt_from_cookies(req: &HttpRequest) -> Result<String, ApiError> {
    req.cookie("auth_token")
        .map(|c| c.value().to_string())
        .ok_or(ApiError::Unauthorized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_service_creation() {
        let _service = JwtService::new("test_secret");
    }
}

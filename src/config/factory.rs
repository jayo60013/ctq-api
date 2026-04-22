use super::EnvConfig;
use crate::services::{GoogleOAuthService, JwtService};

/// Factory for lazily initializing and caching service instances.
/// This avoids recreating expensive-to-construct services on every request.
pub struct ServiceFactory;

impl ServiceFactory {
    /// Creates a `JwtService` from configuration
    pub fn create_jwt_service(config: &EnvConfig) -> JwtService {
        JwtService::new(&config.jwt_secret)
    }

    /// Creates a `GoogleOAuthService` from configuration
    pub fn create_google_oauth_service(config: &EnvConfig) -> GoogleOAuthService {
        GoogleOAuthService::new(
            config.google_client_id.clone(),
            config.google_client_secret.clone(),
            config.google_redirect_uri.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_factory_jwt_creation() {
        let config = EnvConfig {
            database_url: "postgresql://localhost/test".to_string(),
            debug: false,
            allowed_origins: vec!["http://localhost:3000".to_string()],
            google_client_id: "test_id".to_string(),
            google_client_secret: "test_secret".to_string(),
            google_redirect_uri: "http://localhost:3000/callback".to_string(),
            jwt_secret: "test_jwt_secret_abcdefghijklmnop".to_string(),
            secure_cookies: false,
            enable_swagger_ui: false,
        };

        let _jwt_service = ServiceFactory::create_jwt_service(&config);
        // Successfully created without panic
    }

    #[test]
    fn test_service_factory_oauth_creation() {
        let config = EnvConfig {
            database_url: "postgresql://localhost/test".to_string(),
            debug: false,
            allowed_origins: vec!["http://localhost:3000".to_string()],
            google_client_id: "test_id".to_string(),
            google_client_secret: "test_secret".to_string(),
            google_redirect_uri: "http://localhost:3000/callback".to_string(),
            jwt_secret: "test_jwt_secret_abcdefghijklmnop".to_string(),
            secure_cookies: false,
            enable_swagger_ui: false,
        };

        let _oauth_service = ServiceFactory::create_google_oauth_service(&config);
        // Successfully created without panic
    }
}

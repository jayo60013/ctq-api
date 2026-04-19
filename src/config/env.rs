#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub database_url: String,
    pub debug: bool,
    pub allowed_origins: Vec<String>,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
    pub jwt_secret: String,
    pub secure_cookies: bool,
    pub enable_swagger_ui: bool,
}

impl EnvConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")?;

        let debug = std::env::var("DEBUG")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let google_client_id = std::env::var("GOOGLE_CLIENT_ID")?;
        let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")?;
        let google_redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string());

        let jwt_secret = std::env::var("JWT_SECRET")?;

        let secure_cookies = std::env::var("SECURE_COOKIES")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(true);

        let enable_swagger_ui = std::env::var("ENABLE_SWAGGER_UI")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(EnvConfig {
            database_url,
            debug,
            allowed_origins,
            google_client_id,
            google_client_secret,
            google_redirect_uri,
            jwt_secret,
            secure_cookies,
            enable_swagger_ui,
        })
    }
}

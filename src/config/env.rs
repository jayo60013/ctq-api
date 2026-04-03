use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub database_url: String,
    pub start_date: NaiveDate,
    pub debug: bool,
    pub allowed_origins: Vec<String>,
    pub port: u16,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
    pub jwt_secret: String,
    pub cookie_domain: Option<String>,
    pub secure_cookies: bool,
}

impl EnvConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")?;

        let start_date_str =
            std::env::var("START_DATE").unwrap_or_else(|_| "2026-01-01".to_string());
        let start_date = NaiveDate::parse_from_str(&start_date_str, "%Y-%m-%d")?;

        let debug = std::env::var("DEBUG")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(9100);

        let google_client_id = std::env::var("GOOGLE_CLIENT_ID")?;
        let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")?;
        let google_redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:9100/auth/google/callback".to_string());

        let jwt_secret = std::env::var("JWT_SECRET")?;

        let cookie_domain = std::env::var("COOKIE_DOMAIN").ok();

        let secure_cookies = std::env::var("SECURE_COOKIES")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(true);

        Ok(EnvConfig {
            database_url,
            start_date,
            debug,
            allowed_origins,
            port,
            google_client_id,
            google_client_secret,
            google_redirect_uri,
            jwt_secret,
            cookie_domain,
            secure_cookies,
        })
    }
}

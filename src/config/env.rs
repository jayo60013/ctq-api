/// Parse a boolean environment variable, with a default value
fn parse_bool_env(key: &str, default: bool) -> bool {
    std::env::var(key).map_or(default, |v| v.to_lowercase() == "true")
}

/// Validates that a string is not empty
fn validate_non_empty(value: &str, field_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if value.is_empty() {
        return Err(format!("{field_name} cannot be empty").into());
    }
    Ok(())
}

/// Validates that a URL is well-formed
fn validate_url(url: &str, field_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_non_empty(url, field_name)?;
    url::Url::parse(url).map_err(|e| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{field_name} is not a valid URL: {url} ({e})"),
        )) as Box<dyn std::error::Error>
    })?;
    Ok(())
}

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
        validate_url(&database_url, "DATABASE_URL")?;

        let debug = parse_bool_env("DEBUG", false);

        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let google_client_id = std::env::var("GOOGLE_CLIENT_ID")?;
        validate_non_empty(&google_client_id, "GOOGLE_CLIENT_ID")?;

        let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")?;
        validate_non_empty(&google_client_secret, "GOOGLE_CLIENT_SECRET")?;

        let google_redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string());
        validate_url(&google_redirect_uri, "GOOGLE_REDIRECT_URI")?;

        let jwt_secret = std::env::var("JWT_SECRET")?;
        validate_non_empty(&jwt_secret, "JWT_SECRET")?;

        let secure_cookies = parse_bool_env("SECURE_COOKIES", true);
        let enable_swagger_ui = parse_bool_env("ENABLE_SWAGGER_UI", false);

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

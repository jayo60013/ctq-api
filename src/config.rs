use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub start_date: NaiveDate,
    pub debug: bool,
    pub allowed_origins: Vec<String>,
    pub port: u16,
}

impl Config {
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

        Ok(Config {
            database_url,
            start_date,
            debug,
            allowed_origins,
            port,
        })
    }
}

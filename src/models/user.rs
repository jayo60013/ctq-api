use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub google_sub: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPayload {
    pub sub: String,
    pub email: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    #[allow(dead_code)]
    pub id: Uuid,
    #[allow(dead_code)]
    pub email: String,
}

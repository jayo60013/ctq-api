use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::User;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        UserRepository { pool }
    }

    #[allow(dead_code)]
    pub async fn get_by_id(&self, id: Uuid) -> Result<User, ApiError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, google_sub, email, display_name, avatar_url, created_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .ok_or(ApiError::NotFound)?;

        Ok(user)
    }

    #[allow(dead_code)]
    pub async fn get_by_google_sub(&self, google_sub: &str) -> Result<User, ApiError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, google_sub, email, display_name, avatar_url, created_at FROM users WHERE google_sub = $1"
        )
        .bind(google_sub)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        .ok_or(ApiError::NotFound)?;

        Ok(user)
    }

    pub async fn create_or_update(
        &self,
        google_sub: &str,
        email: &str,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<User, ApiError> {
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let user = sqlx::query_as::<_, User>(
            r"
            INSERT INTO users (id, google_sub, email, display_name, avatar_url, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (google_sub) DO UPDATE
            SET email = $3, display_name = $4, avatar_url = $5
            RETURNING id, google_sub, email, display_name, avatar_url, created_at
            ",
        )
        .bind(user_id)
        .bind(google_sub)
        .bind(email)
        .bind(display_name)
        .bind(avatar_url)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(user)
    }
}

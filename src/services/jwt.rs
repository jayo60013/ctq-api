use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::error::ApiError;
use crate::models::{User, UserPayload};

pub struct JwtService {
    secret: Vec<u8>,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        JwtService {
            secret: secret.as_bytes().to_vec(),
        }
    }

    pub fn create_token(&self, user: &User, duration_hours: i64) -> Result<String, ApiError> {
        let now = Utc::now().timestamp();
        let exp = now + (duration_hours * 3600);

        let payload = UserPayload {
            sub: user.id.to_string(),
            email: user.email.clone(),
            iat: now,
            exp,
        };

        let token = encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&self.secret),
        )
        .map_err(|e| ApiError::JwtError(e.to_string()))?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<UserPayload, ApiError> {
        let payload = decode::<UserPayload>(
            token,
            &DecodingKey::from_secret(&self.secret),
            &Validation::default(),
        )
        .map_err(|e| ApiError::JwtError(e.to_string()))?;

        Ok(payload.claims)
    }
}

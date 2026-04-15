use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::engine::Engine;
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use url::Url;

use crate::error::ApiError;
use crate::models::auth::GoogleUserInfo;
use crate::models::{GoogleIdTokenPayload, GoogleTokenResponse};

pub struct GoogleOAuthService {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl GoogleOAuthService {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        GoogleOAuthService {
            client_id,
            client_secret,
            redirect_uri,
        }
    }

    pub fn generate_pkce_pair() -> (String, String) {
        // Generate 128‑char verifier using Alphanumeric + allowed symbols
        let mut rng = rand::thread_rng();
        let code_verifier: String = (0..128).map(|_| rng.sample(Alphanumeric) as char).collect();

        // Compute challenge
        let hash = Sha256::digest(code_verifier.as_bytes());
        let code_challenge = URL_SAFE_NO_PAD.encode(hash);

        (code_verifier, code_challenge)
    }

    pub fn create_auth_url(&self, state: &str, code_challenge: &str) -> String {
        let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap();
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", "openid email profile")
            .append_pair("state", state)
            .append_pair("code_challenge", code_challenge)
            .append_pair("code_challenge_method", "S256");

        url.to_string()
    }

    pub async fn exchange_code_for_token(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<(String, String), ApiError> {
        let client = reqwest::Client::new();

        let body = format!(
            "client_id={}&client_secret={}&code={}&code_verifier={}&grant_type=authorization_code&redirect_uri={}",
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.client_secret),
            urlencoding::encode(code),
            urlencoding::encode(code_verifier),
            urlencoding::encode(&self.redirect_uri),
        );

        let response = client
            .post("https://oauth2.googleapis.com/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| {
                ApiError::ExternalServiceError(format!("Google token request failed: {e}"))
            })?;

        let token_response: GoogleTokenResponse = response.json().await.map_err(|e| {
            ApiError::ExternalServiceError(format!("Failed to parse token response: {e}"))
        })?;

        Ok((token_response.id_token, token_response.access_token))
    }

    pub async fn verify_id_token(
        &self,
        id_token: &str,
        access_token: &str,
    ) -> Result<GoogleIdTokenPayload, ApiError> {
        let client = reqwest::Client::new();

        let url = format!("https://www.googleapis.com/oauth2/v1/tokeninfo?id_token={id_token}");

        let response = client.get(&url).send().await.map_err(|e| {
            ApiError::ExternalServiceError(format!("Token verification request failed: {e}"))
        })?;

        if !response.status().is_success() {
            return Err(ApiError::ExternalServiceError(
                "Token verification failed".to_string(),
            ));
        }

        let text = response.text().await.map_err(|e| {
            ApiError::ExternalServiceError(format!("Failed to read response body: {e}"))
        })?;

        tracing::debug!("Google tokeninfo response: {}", text);

        let mut payload: GoogleIdTokenPayload = serde_json::from_str(&text).map_err(|e| {
            tracing::error!(
                "Failed to parse token payload. Response body: {}. Error: {}",
                text,
                e
            );
            ApiError::ExternalServiceError(format!("Failed to parse token payload: {e}"))
        })?;

        if payload.audience != self.client_id {
            return Err(ApiError::ExternalServiceError(
                "Invalid audience in token".to_string(),
            ));
        }

        // Fetch user profile info separately to get name and picture
        // The tokeninfo endpoint doesn't return these fields, but the userinfo endpoint does
        if let Ok(userinfo) = self.get_userinfo(access_token).await {
            if payload.name.is_none() {
                // Prefer given_name over full name for display
                payload.name = userinfo.given_name.or(userinfo.name);
            }
            if payload.picture.is_none() {
                payload.picture = userinfo.picture;
            }
        }

        Ok(payload)
    }

    async fn get_userinfo(&self, access_token: &str) -> Result<GoogleUserInfo, ApiError> {
        let client = reqwest::Client::new();

        let url =
            format!("https://www.googleapis.com/oauth2/v1/userinfo?access_token={access_token}");

        tracing::debug!("Fetching userinfo from: {}", url);

        let response = client.get(&url).send().await.map_err(|e| {
            ApiError::ExternalServiceError(format!("User info request failed: {e}"))
        })?;

        if !response.status().is_success() {
            return Err(ApiError::ExternalServiceError(format!(
                "Failed to fetch user info. Status: {}",
                response.status()
            )));
        }

        let text = response.text().await.map_err(|e| {
            ApiError::ExternalServiceError(format!("Failed to read userinfo response: {e}"))
        })?;

        tracing::debug!("Google userinfo response: {}", text);

        let userinfo: GoogleUserInfo = serde_json::from_str(&text).map_err(|e| {
            tracing::error!(
                "Failed to parse userinfo payload. Response body: {}. Error: {}",
                text,
                e
            );
            ApiError::ExternalServiceError(format!("Failed to parse userinfo payload: {e}"))
        })?;

        Ok(userinfo)
    }
}

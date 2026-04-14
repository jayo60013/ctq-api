use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GoogleAuthUrl {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleTokenResponse {
    pub id_token: String,
    #[allow(dead_code)]
    pub access_token: String,
    #[allow(dead_code)]
    pub expires_in: i32,
}

#[derive(Debug, Deserialize)]
pub struct GoogleIdTokenPayload {
    pub user_id: String,
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub picture: Option<String>,
    pub audience: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub expires_in: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub email_verified: Option<bool>,
    #[serde(default)]
    #[allow(dead_code)]
    pub issuer: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub user_id: String,
    pub email: String,
}

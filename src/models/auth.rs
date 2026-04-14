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
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub aud: String,
    #[allow(dead_code)]
    pub exp: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub user_id: String,
    pub email: String,
}

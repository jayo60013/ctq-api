use actix_web::{
    HttpResponse, Responder, post,
    web::{self, Data},
};
use serde_json::json;
use sqlx::PgPool;

use crate::auth::{
    model::user::{SignupPayload, User},
    service::password::hash_password,
};
use log::error;

#[post("/login")]
pub async fn login(pool: Data<PgPool>, user: web::Json<User>) -> impl Responder {
    HttpResponse::NotImplemented()
}

#[post("/signup")]
pub async fn signup(pool: Data<PgPool>, body: web::Json<SignupPayload>) -> impl Responder {
    let payload = body.into_inner();

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let res = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, password_hash)
        VALUES ($1, $2)
        RETURNING id, username, created_at
        "#,
    )
    .bind(payload.username) // CITEXT will handle case-insensitive uniqueness
    .bind(&hash_password(&payload.password))
    .fetch_one(&mut *tx)
    .await;

    let user = match res {
        Ok(u) => u,
        Err(e) => {
            // Unique violation => 409 Conflict
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().map(|c| c.as_ref()) == Some("23505") {
                    return HttpResponse::Conflict()
                        .json(json!({"error": "Account already exists"}));
                }
            }
            return HttpResponse::InternalServerError().finish();
        }
    };

    if tx.commit().await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    let location = format!("/users/{}", user.id);
    HttpResponse::Created()
        .append_header(("Location", location))
        .json(user)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(login).service(signup);
}

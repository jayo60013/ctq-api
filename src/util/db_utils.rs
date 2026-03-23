use sqlx::{Pool, Postgres};

pub async fn connect_pool() -> anyhow::Result<Pool<Postgres>> {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL")?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    Ok(pool)
}

mod config;
mod db;
mod error;
mod health;
mod models;
mod puzzle_cache;
mod routes;
mod services;
mod transformer;
mod validators;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use sqlx::postgres::PgPoolOptions;

use config::Config;
use puzzle_cache::DailyPuzzleCache;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env().expect("Failed to load configuration");

    let log_level = if config.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("ctq_api={log_level},actix_web=info")
                    .parse()
                    .unwrap()
            }),
        )
        .with_writer(std::io::stdout)
        .init();

    tracing::info!("Starting Crack the Quote API");
    tracing::debug!("Config: {:?}", config);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to verify database connection");

    tracing::info!("Connected to Postgres");

    let pool = web::Data::new(pool);
    let daily_puzzle_cache = web::Data::new(DailyPuzzleCache::new());
    let server_port = config.port;

    tracing::info!("Starting HTTP server on 0.0.0.0:{}", server_port);

    HttpServer::new(move || {
        let allowed_origins = config.allowed_origins.clone();
        let config_data = web::Data::new(config.clone());
        let cors = Cors::default()
            .allowed_origin_fn(move |origin, _req_head| {
                allowed_origins
                    .iter()
                    .any(|allowed| origin.as_bytes().eq_ignore_ascii_case(allowed.as_bytes()))
            })
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(pool.clone())
            .app_data(config_data)
            .app_data(daily_puzzle_cache.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .configure(routes::init_routes)
            .configure(health::init)
    })
    .bind(("0.0.0.0", server_port))?
    .run()
    .await
}

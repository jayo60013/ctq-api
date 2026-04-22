mod config;
mod error;
mod health;
mod middleware;
mod models;
mod openapi;
mod puzzle_cache;
mod repository;
mod routes;
mod services;
mod transformer;
mod validators;

use actix_web::{middleware::Logger, web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use config::{EnvConfig, ServiceFactory};
use middleware::create_cors;
use openapi::ApiDoc;
use puzzle_cache::DailyPuzzleCache;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = EnvConfig::from_env().expect("Failed to load configuration");

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

    // Initialize cached services at startup
    let jwt_service = web::Data::new(ServiceFactory::create_jwt_service(&config));
    let oauth_service = web::Data::new(ServiceFactory::create_google_oauth_service(&config));

    let pool = web::Data::new(pool);
    let daily_puzzle_cache = web::Data::new(DailyPuzzleCache::new());
    let config_data = web::Data::new(config.clone());
    let server_port: u16 = 9100;

    tracing::info!("Starting HTTP server on 0.0.0.0:{}", server_port);

    HttpServer::new(move || {
        let allowed_origins = config.allowed_origins.clone();
        let cors = create_cors(&allowed_origins);

        let app = App::new()
            .app_data(pool.clone())
            .app_data(config_data.clone())
            .app_data(daily_puzzle_cache.clone())
            .app_data(jwt_service.clone())
            .app_data(oauth_service.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .configure(routes::init_routes)
            .configure(health::init);

        if config.enable_swagger_ui {
            app.service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
        } else {
            app
        }
    })
    .bind(("0.0.0.0", server_port))?
    .run()
    .await
}

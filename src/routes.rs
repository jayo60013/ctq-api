// Route modules
pub mod archive;
pub mod daily;

use actix_web::web;

/// Initialize all API routes
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/puzzles")
            .configure(daily::init)
            .configure(archive::init),
    );
}

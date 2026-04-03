pub mod auth;
pub mod puzzles;
pub mod me;

use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.configure(puzzles::init)
        .configure(auth::init)
        .configure(me::init);
}

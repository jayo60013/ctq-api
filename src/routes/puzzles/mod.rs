pub mod archive;
pub mod daily;

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/puzzles")
            .configure(daily::init)
            .configure(archive::init),
    );
}

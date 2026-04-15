pub mod activities;
pub mod stats;

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/me/activities").configure(activities::init))
        .service(actix_web::web::scope("/me").configure(stats::init));
}

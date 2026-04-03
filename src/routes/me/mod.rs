pub mod activities;

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/me/activities")
            .configure(activities::init),
    );
}


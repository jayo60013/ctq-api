pub mod google;
pub mod logout;

pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/auth")
            .configure(google::init)
            .configure(logout::init),
    );
}

use actix_cors::Cors;
use actix_web::http;

pub fn create_cors(allowed_origins: &[String]) -> Cors {
    let mut cors = Cors::default();

    for origin in allowed_origins {
        cors = cors.allowed_origin(origin);
    }

    cors.allowed_methods(vec![
        http::Method::GET,
        http::Method::POST,
        http::Method::OPTIONS,
    ])
    .allowed_headers(vec![http::header::CONTENT_TYPE])
    .supports_credentials()
    .max_age(3600)
}

use actix_web::{web, HttpResponse};
use uuid::Uuid;
use web::ServiceConfig;

use crate::repository::RepositoryInjector;

const PATH: &str = "/user";

pub fn service(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(PATH)
            // GET
            .route("/{user_id}", web::get().to(get)),
    );
}

async fn get(user_id: web::Path<String>, repo: web::Data<RepositoryInjector>) -> HttpResponse {
    if let Ok(parsed_user_id) = Uuid::parse_str(&user_id) {
        match repo.get_user(&parsed_user_id) {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(_) => HttpResponse::NotFound().body("Not found"),
        }
    } else {
        HttpResponse::BadRequest().body("Invalid UUID")
    }
}

use actix_web::{
    error::PathError,
    web::{self, PathConfig},
    HttpRequest, HttpResponse,
};
use uuid::Uuid;
use web::ServiceConfig;

use crate::repository::RepositoryInjector;

const PATH: &str = "/user";

pub fn service(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(PATH)
            .app_data(PathConfig::default().error_handler(path_config_handler))
            // GET
            .route("/{user_id}", web::get().to(get)),
    );
}

async fn get(user_id: web::Path<Uuid>, repo: RepositoryInjector) -> HttpResponse {
    match repo.get_user(&user_id) {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().body("Not found"),
    }
}

fn path_config_handler(err: PathError, _req: &HttpRequest) -> actix_web::Error {
    actix_web::error::ErrorBadRequest(err)
}

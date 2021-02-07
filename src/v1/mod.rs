mod users;

use actix_web::web::{self, ServiceConfig};

pub fn service(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/v1").configure(users::service));
}

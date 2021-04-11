mod users;

use crate::repository::Repository;
use actix_web::web::{self, ServiceConfig};

pub fn service<R: Repository>(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/v1").configure(users::service::<R>));
}

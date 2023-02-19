mod health;
pub mod repository;
mod user;
mod v1;

use actix_web::web::{self, ServiceConfig};
use std::sync::{
    atomic::{AtomicU16, Ordering},
    Arc,
};

pub fn start<R: repository::Repository + 'static>(
    repo: web::Data<R>,
    thread_counter: Arc<AtomicU16>,
    cfg: &mut ServiceConfig,
) {
    let thread_index = thread_counter.fetch_add(1, Ordering::SeqCst);
    tracing::trace!("Starting thread {}", thread_index);

    cfg.app_data(web::Data::new(thread_index))
        .app_data(repo)
        .configure(v1::service::<R>)
        .configure(health::service);
}

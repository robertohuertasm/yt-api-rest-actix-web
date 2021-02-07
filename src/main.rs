mod health;
mod repository;
mod user;
mod v1;

use actix_web::{web, App, HttpServer};
use repository::{MemoryRepository, RepositoryInjector};
use std::sync::{
    atomic::{AtomicU16, Ordering},
    Arc,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // init env vars
    dotenv::dotenv().ok();
    // building address
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("127.0.0.1:{}", port);
    // building shared state
    println!("Starting our server");
    let thread_counter = Arc::new(AtomicU16::new(1));
    let repo = RepositoryInjector::new(MemoryRepository::default());
    let repo = web::Data::new(repo);
    // starting the server
    HttpServer::new(move || {
        let thread_index = thread_counter.fetch_add(1, Ordering::SeqCst);
        println!("Starting thread {}", thread_index);
        // starting the services
        App::new()
            .data(thread_index)
            .app_data(repo.clone())
            .configure(v1::service)
            .configure(health::service)
    })
    .bind(&address)
    .unwrap_or_else(|err| {
        panic!(
            "🔥🔥🔥 Couldn't start the server in port {}: {:?}",
            port, err
        )
    })
    .run()
    .await
}

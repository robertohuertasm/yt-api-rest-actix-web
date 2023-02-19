use actix_web::{web, App, HttpServer};
use mi_api::{repository::PostgresRepository, start};
use std::sync::{atomic::AtomicU16, Arc};
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // init env vars
    dotenv::dotenv().ok();
    // init tracing subscriber
    let tracing = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .with_env_filter(EnvFilter::from_default_env());

    if cfg!(debug_assertions) {
        tracing.pretty().init();
    } else {
        tracing.json().init();
    }

    // building address
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("127.0.0.1:{}", port);
    // building shared state
    tracing::debug!("Starting our server at {}", address);
    let thread_counter = Arc::new(AtomicU16::new(1));
    let repo = PostgresRepository::from_env()
        .await
        .expect("Repository initialization error");
    let repo = web::Data::new(repo);

    // starting the server
    HttpServer::new(move || {
        let repo = repo.clone();
        let thread_counter = Arc::clone(&thread_counter);
        App::new().configure(|cfg| {
            start(repo, thread_counter, cfg);
        })
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

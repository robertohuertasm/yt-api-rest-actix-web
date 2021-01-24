mod repository;
mod user;

use std::sync::{
    atomic::{AtomicU16, Ordering},
    Arc,
};

use actix_web::{web, App, HttpResponse, HttpServer};
use repository::{MemoryRepository, Repository};
use uuid::Uuid;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // init env vars
    dotenv::dotenv().ok();
    // building address
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("127.0.0.1:{}", port);

    println!("Starting our server");
    let thread_counter = Arc::new(AtomicU16::new(1));
    let repo = Arc::new(MemoryRepository::default());

    HttpServer::new(move || {
        let thread_index = thread_counter.fetch_add(1, Ordering::SeqCst);
        println!("Starting thread {}", thread_index);

        App::new()
            .data(thread_index)
            .data(repo.clone())
            .route("/", web::get().to(|| HttpResponse::Ok().body("Hola Rust")))
            .service(web::resource("/user/{user_id}").route(web::get().to(get_user)))
            .route(
                "/health",
                web::get().to(|index: web::Data<u16>| {
                    HttpResponse::Ok()
                        .header("thread-id", index.to_string())
                        .finish()
                }),
            )
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

async fn get_user(
    user_id: web::Path<String>,
    repo: web::Data<Arc<MemoryRepository>>,
) -> HttpResponse {
    if let Ok(parsed_user_id) = Uuid::parse_str(&user_id) {
        match repo.get_user(&parsed_user_id) {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(_) => HttpResponse::NotFound().body("Not found"),
        }
    } else {
        HttpResponse::BadRequest().body("Invalid UUID")
    }
}

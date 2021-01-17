mod repository;
mod user;

use std::sync::{
    atomic::{AtomicU16, Ordering},
    Arc,
};

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use repository::{MemoryRepository, Repository};
use uuid::Uuid;

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("mundo");
    format!("Hola {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let address = format!("127.0.0.1:{}", port);

    println!("Starting our server");
    let thread_counter = Arc::new(AtomicU16::new(1));

    HttpServer::new(move || {
        println!(
            "Starting thread {}",
            thread_counter.fetch_add(1, Ordering::SeqCst)
        );
        let thread_index = thread_counter.load(Ordering::SeqCst);
        App::new()
            .service(web::resource("/user/{user_id}").route(web::get().to(get_user)))
            .route("/", web::get().to(greet))
            .route(
                "/health",
                web::get().to(move || {
                    HttpResponse::Ok()
                        .header("thread-id", thread_index.to_string())
                        .finish()
                }),
            )
            .route("/str", web::get().to(|| async { "Hola Rust" }))
            .route("/{name}", web::get().to(greet))
    })
    .bind(&address)?
    .run()
    .await
}

async fn get_user(user_id: web::Path<String>) -> HttpResponse {
    if let Ok(parsed_user_id) = Uuid::parse_str(&user_id) {
        let repo = MemoryRepository::default();
        match repo.get_users(&parsed_user_id) {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(_) => HttpResponse::NotFound().body("Not found"),
        }
    } else {
        HttpResponse::BadRequest().body("Invalid UUID")
    }
}

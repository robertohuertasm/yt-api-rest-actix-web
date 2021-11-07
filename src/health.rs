use actix_web::{
    web::{self, ServiceConfig},
    HttpResponse,
};
use tracing::instrument;

#[instrument(skip(cfg), level = "trace")]
pub fn service(cfg: &mut ServiceConfig) {
    tracing::trace!("Init health service");
    cfg.route("/health", web::get().to(health_check));
}

#[instrument]
async fn health_check(index: web::Data<u16>) -> HttpResponse {
    HttpResponse::Ok()
        .append_header(("thread-id", index.to_string()))
        .finish()
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, App};

    use super::*;

    #[actix_rt::test]
    async fn health_check_works() {
        let res = health_check(web::Data::new(5)).await;
        assert!(res.status().is_success());
        assert_eq!(res.status(), StatusCode::OK);
        let data = res
            .headers()
            .get("thread-id")
            .map(|h| h.to_str().ok())
            .flatten();
        assert_eq!(data, Some("5"));
    }

    #[actix_rt::test]
    async fn health_check_integration_works() {
        let app = App::new().app_data(web::Data::new(5u16)).configure(service);
        let mut app = actix_web::test::init_service(app).await;
        let req = actix_web::test::TestRequest::get()
            .uri("/health")
            .to_request();
        let res = actix_web::test::call_service(&mut app, req).await;
        assert!(res.status().is_success());
        assert_eq!(res.status(), StatusCode::OK);
        let data = res
            .headers()
            .get("thread-id")
            .map(|h| h.to_str().ok())
            .flatten();
        assert_eq!(data, Some("5"));
    }
}

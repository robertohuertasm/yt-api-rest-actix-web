use actix_web::{
    error::PathError,
    web::{self, PathConfig},
    HttpRequest, HttpResponse,
};
use uuid::Uuid;
use web::ServiceConfig;

use crate::repository::Repository;

const PATH: &str = "/user";

pub fn service<R: Repository>(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope(PATH)
            .app_data(PathConfig::default().error_handler(path_config_handler))
            // GET
            .route("/{user_id}", web::get().to(get::<R>)),
    );
}

async fn get<R: Repository>(user_id: web::Path<Uuid>, repo: web::Data<R>) -> HttpResponse {
    match repo.get_user(&user_id) {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().body("Not found"),
    }
}

fn path_config_handler(err: PathError, _req: &HttpRequest) -> actix_web::Error {
    actix_web::error::ErrorBadRequest(err)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::user::User;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        CustomRepo {}
        impl Repository for CustomRepo {
            fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, String>;
        }
    }

    #[actix_rt::test]
    async fn it_works() {
        let user_id = uuid::Uuid::new_v4();
        let user_name = "Mi nombre";

        let mut repo = MockCustomRepo::default();
        repo.expect_get_user().returning(move |id| {
            let mut user = User::new(user_name.to_string(), (1977, 03, 10));
            user.id = *id;
            Ok(user)
        });

        let mut result = get(web::Path::from(user_id), web::Data::new(repo)).await;

        let user = result
            .take_body()
            .as_ref()
            .map(|b| match b {
                actix_web::dev::Body::Bytes(x) => serde_json::from_slice::<'_, User>(x).ok(),
                _ => None,
            })
            .flatten()
            .unwrap();

        assert_eq!(user.id, user_id);
        assert_eq!(user.name, user_name);
    }
}

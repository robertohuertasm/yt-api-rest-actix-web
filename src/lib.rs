use std::sync::{atomic::AtomicU16, Arc};

use actix_web::web::{self, ServiceConfig};
use mi_api::{repository::PostgresRepository, start};
use shuttle_service::{async_trait, Factory, ResourceBuilder, Runtime, ShuttleActixWeb};

struct Repository {}

#[async_trait]
impl ResourceBuilder<web::Data<PostgresRepository>> for Repository {
    fn new() -> Self {
        Self {}
    }

    async fn build(
        self,
        factory: &mut dyn Factory,
        runtime: &Runtime,
    ) -> Result<web::Data<PostgresRepository>, shuttle_service::Error> {
        let conn_str = match factory.get_environment() {
            shuttle_service::Environment::Local => {
                let secrets = factory.get_secrets().await?;
                let pwd = secrets
                    .get("DB_PASSWORD")
                    .ok_or(shuttle_service::Error::Secret(
                        "DB_PASSWORD_NOT_FOUND".to_string(),
                    ))?;
                format!("postgres://postgres:{pwd}@localhost:5432/rpts")
            }
            shuttle_service::Environment::Production => {
                factory
                    .get_db_connection_string(shuttle_service::database::Type::Shared(
                        shuttle_service::database::SharedEngine::Postgres,
                    ))
                    .await?
            }
        };

        let pool = shuttle_shared_db::Postgres::new()
            .local_uri(&conn_str)
            .build(factory, runtime)
            .await?;

        Ok(web::Data::new(PostgresRepository::from_pool(pool)))
    }
}

struct Counter {}

#[async_trait]
impl ResourceBuilder<Arc<AtomicU16>> for Counter {
    fn new() -> Self {
        Self {}
    }

    async fn build(
        self,
        _factory: &mut dyn Factory,
        _runtime: &Runtime,
    ) -> Result<Arc<AtomicU16>, shuttle_service::Error> {
        Ok(Arc::new(AtomicU16::new(1)))
    }
}

#[shuttle_service::main]
async fn actix_web(
    #[Repository] repo: web::Data<PostgresRepository>,
    #[Counter] thread_counter: Arc<AtomicU16>,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Sync + Send + Clone + 'static> {
    Ok(move |cfg: &mut ServiceConfig| {
        tracing::info!("SHUTTLE: Starting our server thread");
        start(repo, thread_counter, cfg);
    })
}

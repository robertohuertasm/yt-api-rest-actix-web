use actix_web::FromRequest;

use crate::user::User;
use std::{
    future::{ready, Ready},
    ops::Deref,
    sync::Arc,
};

pub trait Repository: Send + Sync + 'static {
    fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, String>;
}
pub struct RepositoryInjector(Arc<Box<dyn Repository>>);

impl RepositoryInjector {
    pub fn new(repo: impl Repository) -> Self {
        Self(Arc::new(Box::new(repo)))
    }
}

impl Clone for RepositoryInjector {
    fn clone(&self) -> Self {
        let repo = self.0.clone();
        Self(repo)
    }
}

impl Deref for RepositoryInjector {
    type Target = dyn Repository;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().as_ref()
    }
}

impl FromRequest for RepositoryInjector {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        if let Some(injector) = req.app_data::<Self>() {
            let owned_injector = injector.to_owned();
            ready(Ok(owned_injector))
        } else {
            ready(Err(actix_web::error::ErrorBadRequest(
                "No repository injector was found in the request",
            )))
        }
    }
}

pub struct MemoryRepository {
    users: Vec<User>,
}

impl Default for MemoryRepository {
    fn default() -> Self {
        Self {
            users: vec![User::new("Rob".to_string(), (1977, 03, 10))],
        }
    }
}

impl Repository for MemoryRepository {
    fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, String> {
        self.users
            .iter()
            .find(|u| &u.id == user_id)
            .cloned()
            .ok_or_else(|| "Invalid UUID".to_string())
    }
}

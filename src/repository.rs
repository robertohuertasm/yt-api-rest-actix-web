use async_trait::async_trait;
use chrono::Utc;
use std::sync::{PoisonError, RwLock};
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;

use crate::user::User;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("PoisonError: `{0}`")]
    LockError(String),
    #[error("This entity already exists")]
    AlreadyExists,
    #[error("This entity does not exist")]
    DoesNotExist,
    #[error("The id format is not valid")]
    InvalidId,
}

impl<T> From<PoisonError<T>> for RepositoryError {
    fn from(poison_error: PoisonError<T>) -> Self {
        RepositoryError::LockError(poison_error.to_string())
    }
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn get_user(&self, user_id: &Uuid) -> RepositoryResult<User>;
    async fn create_user(&self, user: &User) -> RepositoryResult<User>;
    async fn update_user(&self, user: &User) -> RepositoryResult<User>;
    async fn delete_user(&self, user_id: &Uuid) -> RepositoryResult<Uuid>;
}

pub struct MemoryRepository {
    users: RwLock<Vec<User>>,
}

impl Default for MemoryRepository {
    fn default() -> Self {
        Self {
            users: RwLock::new(vec![]),
        }
    }
}

#[async_trait]
impl Repository for MemoryRepository {
    #[instrument(skip(self))]
    async fn get_user(&self, user_id: &uuid::Uuid) -> RepositoryResult<User> {
        let users = self.users.read()?;
        let result = users
            .iter()
            .find(|u| &u.id == user_id)
            .cloned()
            .ok_or_else(|| RepositoryError::InvalidId);

        if result.is_err() {
            tracing::error!("Couldn't retrive a user with id {}", user_id);
        }

        result
    }

    #[instrument(skip(self))]
    async fn create_user(&self, user: &User) -> RepositoryResult<User> {
        if self.get_user(&user.id).await.is_ok() {
            tracing::error!("User with id {} already exists", user.id);
            return Err(RepositoryError::AlreadyExists);
        }
        let mut new_user = user.to_owned();
        new_user.created_at = Some(Utc::now());
        let mut users = self.users.write().unwrap();
        users.push(new_user.clone());
        tracing::trace!("User with id {} correctly created", user.id);
        Ok(new_user)
    }

    #[instrument(skip(self))]
    async fn update_user(&self, user: &User) -> RepositoryResult<User> {
        if let Ok(old_user) = self.get_user(&user.id).await {
            let mut updated_user = user.to_owned();
            updated_user.created_at = old_user.created_at;
            updated_user.updated_at = Some(Utc::now());
            let mut users = self.users.write().unwrap();
            users.retain(|x| x.id != user.id);
            users.push(updated_user.clone());
            tracing::debug!("User with id {} correctly updated", user.id);
            Ok(updated_user)
        } else {
            tracing::error!("User {} does not exit", user.id);
            Err(RepositoryError::DoesNotExist)
        }
    }

    #[instrument(skip(self), err)]
    async fn delete_user(&self, user_id: &Uuid) -> RepositoryResult<Uuid> {
        let mut users = self.users.write()?;
        users.retain(|x| &x.id != user_id);
        Ok(user_id.to_owned())
    }
}

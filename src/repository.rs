use chrono::Utc;
use futures::{future::BoxFuture, FutureExt};
use std::sync::{PoisonError, RwLock};
use thiserror::Error;
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

type RepositoryResultOutput<T> = Result<T, RepositoryError>;
type RepositoryResult<'a, T> = BoxFuture<'a, RepositoryResultOutput<T>>;

pub trait Repository: Send + Sync + 'static {
    fn get_user<'a>(&'a self, user_id: &'a Uuid) -> RepositoryResult<'a, User>;
    fn create_user<'a>(&'a self, user: &'a User) -> RepositoryResult<'a, User>;
    fn update_user<'a>(&'a self, user: &'a User) -> RepositoryResult<'a, User>;
    fn delete_user<'a>(&'a self, user_id: &'a Uuid) -> RepositoryResult<'a, Uuid>;
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

impl Repository for MemoryRepository {
    fn get_user<'a>(&'a self, user_id: &'a uuid::Uuid) -> RepositoryResult<'a, User> {
        async move {
            let users = self.users.read()?;
            users
                .iter()
                .find(|u| &u.id == user_id)
                .cloned()
                .ok_or_else(|| RepositoryError::InvalidId)
        }
        .boxed()
    }

    fn create_user<'a>(&'a self, user: &'a User) -> RepositoryResult<'a, User> {
        async move {
            if self.get_user(&user.id).await.is_ok() {
                return Err(RepositoryError::AlreadyExists);
            }
            let mut new_user = user.to_owned();
            new_user.created_at = Some(Utc::now());
            let mut users = self.users.write().unwrap();
            users.push(new_user.clone());
            Ok(new_user)
        }
        .boxed()
    }

    fn update_user<'a>(&'a self, user: &'a User) -> RepositoryResult<'a, User> {
        async move {
            if let Ok(old_user) = self.get_user(&user.id).await {
                let mut updated_user = user.to_owned();
                updated_user.created_at = old_user.created_at;
                updated_user.updated_at = Some(Utc::now());
                let mut users = self.users.write().unwrap();
                users.retain(|x| x.id != user.id);
                users.push(updated_user.clone());
                Ok(updated_user)
            } else {
                Err(RepositoryError::DoesNotExist)
            }
        }
        .boxed()
    }

    fn delete_user<'a>(&'a self, user_id: &'a Uuid) -> RepositoryResult<'a, Uuid> {
        async move {
            let mut users = self.users.write()?;
            users.retain(|x| &x.id != user_id);
            Ok(user_id.to_owned())
        }
        .boxed()
    }
}

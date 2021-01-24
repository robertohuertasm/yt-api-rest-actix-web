use crate::user::User;

pub trait Repository {
    fn get_user(&self, user_id: &uuid::Uuid) -> Result<User, String>;
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

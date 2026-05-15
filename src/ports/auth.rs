use anyhow::Result;
use uuid::Uuid;

pub trait AuthProvider: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String>;
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool>;
    fn generate_token(&self, user_id: Uuid) -> Result<String>;
}

use crate::domain::user::{User, UserRole};
use crate::ports::repository::UserRepository;
use crate::ports::auth::AuthProvider;
use uuid::Uuid;
use anyhow::Result;

pub struct UserService {
    user_repo: Box<dyn UserRepository>,
    auth_provider: Box<dyn AuthProvider>,
}

impl UserService {
    pub fn new(user_repo: Box<dyn UserRepository>, auth_provider: Box<dyn AuthProvider>) -> Self {
        Self { user_repo, auth_provider }
    }

    pub async fn register_user(
        &self,
        username: String,
        email: String,
        password: String,
        full_name: String,
    ) -> Result<User> {
        // Check if username exists
        if self.user_repo.find_by_username(&username)?.is_some() {
            return Err(anyhow::anyhow!("Username already exists"));
        }

        // Check if email exists
        if self.user_repo.find_by_email(&email)?.is_some() {
            return Err(anyhow::anyhow!("Email already exists"));
        }

        // Hash password
        let password_hash = self.auth_provider.hash_password(&password)?;

        // Create user
        let user = User {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            full_name,
            role: UserRole::User,
            manager_id: None,
            is_active: true,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            last_login: None,
        };

        let _user_id = self.user_repo.create(user.clone())?;
        Ok(user)
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Result<(User, String)> {
        let user = self.user_repo
            .find_by_username(username)?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        if !user.is_active {
            return Err(anyhow::anyhow!("User is inactive"));
        }

        let is_valid = self.auth_provider.verify_password(password, &user.password_hash)?;
        if !is_valid {
            return Err(anyhow::anyhow!("Invalid password"));
        }

        // Update last login
        self.user_repo.update_last_login(user.id)?;

        // Generate token
        let token = self.auth_provider.generate_token(user.id)?;

        Ok((user, token))
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        self.user_repo.find_by_id(user_id)
    }

    pub async fn update_user(&self, user: User) -> Result<()> {
        self.user_repo.update(user)
    }

    pub async fn get_subordinates(&self, _manager_id: Uuid) -> Result<Vec<User>> {
        // This would need to be implemented in TaskRepository or added to UserRepository
        // For now, return empty
        Ok(vec![])
    }
}

use crate::domain::user::{User, UserRole};
use crate::ports::repository::UserRepository;
use crate::ports::repository::EventLogRepository;
use crate::ports::auth::AuthProvider;
use uuid::Uuid;
use anyhow::Result;

pub struct UserService {
    user_repo: Box<dyn UserRepository>,
    auth_provider: Box<dyn AuthProvider>,
    event_log_repo: Box<dyn EventLogRepository>,
}

impl UserService {
    pub fn new(user_repo: Box<dyn UserRepository>, auth_provider: Box<dyn AuthProvider>, event_log_repo: Box<dyn EventLogRepository>) -> Self {
        Self { user_repo, auth_provider, event_log_repo }
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

        let user_id = self.user_repo.create(user.clone())?;
        self.log_event(user_id, "create", "user", &user_id.to_string(), None)?;
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
        self.log_event(user.id, "login", "user", &user.id.to_string(), None)?;

        // Generate token
        let token = self.auth_provider.generate_token(user.id)?;

        Ok((user, token))
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        self.user_repo.find_by_id(user_id)
    }

    pub async fn update_user(&self, user: User) -> Result<()> {
        self.user_repo.update(user.clone())?;
        self.log_event(user.id, "update", "user", &user.id.to_string(), None)?;
        Ok(())
    }

    pub async fn get_subordinates(&self, _manager_id: Uuid) -> Result<Vec<User>> {
        // This would need to be implemented in TaskRepository or added to UserRepository
        // For now, return empty
        Ok(vec![])
    }

    pub async fn get_workload(&self, user_id: Uuid) -> Result<i32> {
        self.user_repo.get_workload(user_id)
    }

    fn log_event(
        &self,
        user_id: Uuid,
        event_type: &str,
        entity_type: &str,
        entity_id: &str,
        workspace_id: Option<Uuid>,
    ) -> Result<()> {
        use crate::domain::audit::EventLog;
        let event = EventLog {
            id: 0,
            user_id,
            event_type: event_type.to_string(),
            entity_type: Some(entity_type.to_string()),
            entity_id: Some(entity_id.to_string()),
            data: serde_json::json!({}),
            previous_value: None,
            new_value: None,
            workspace_id,
            created_at: chrono::Utc::now(),
        };
        self.event_log_repo.log(event)?;
        Ok(())
    }
}

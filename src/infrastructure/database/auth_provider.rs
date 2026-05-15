use crate::ports::auth::AuthProvider;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use jsonwebtoken::errors::Result as JwtResult;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{Duration, Utc};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub struct Argon2AuthProvider {
    jwt_secret: String,
}

impl Argon2AuthProvider {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }
}

impl AuthProvider for Argon2AuthProvider {
    fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?;
        Ok(password_hash.to_string())
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    fn generate_token(&self, user_id: Uuid) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(8);
        
        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;
        
        Ok(token)
    }
}

pub fn verify_token(token: &str, jwt_secret: &str) -> JwtResult<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

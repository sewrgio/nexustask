use nexustask::infrastructure::database::auth_provider::{Argon2AuthProvider, verify_token};
use nexustask::ports::auth::AuthProvider;
use uuid::Uuid;

#[test]
fn test_password_hashing() {
    let auth_provider = Argon2AuthProvider::new("test_secret".to_string());
    let password = "test_password_123";
    
    let hashed = auth_provider.hash_password(password).expect("Failed to hash password");
    
    // Hash should be different from original password
    assert_ne!(password, hashed);
    
    // Hash should be consistent for same password (but different due to salt)
    let hashed2 = auth_provider.hash_password(password).expect("Failed to hash password");
    assert_ne!(hashed, hashed2); // Argon2 uses salt, so hashes should differ
    
    // Verify should work
    assert!(auth_provider.verify_password(password, &hashed).expect("Failed to verify password"));
}

#[test]
fn test_password_verification_failure() {
    let auth_provider = Argon2AuthProvider::new("test_secret".to_string());
    let password = "test_password_123";
    let wrong_password = "wrong_password";
    
    let hashed = auth_provider.hash_password(password).expect("Failed to hash password");
    
    // Wrong password should fail verification
    assert!(!auth_provider.verify_password(wrong_password, &hashed).expect("Failed to verify password"));
}

#[test]
fn test_jwt_token_generation() {
    let auth_provider = Argon2AuthProvider::new("test_secret_key".to_string());
    let user_id = Uuid::new_v4();
    
    let token = auth_provider.generate_token(user_id).expect("Failed to generate token");
    
    // Token should not be empty
    assert!(!token.is_empty());
    
    // Token should have 3 parts (header.payload.signature)
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3);
}

#[test]
fn test_jwt_token_verification() {
    let auth_provider = Argon2AuthProvider::new("test_secret_key".to_string());
    let user_id = Uuid::new_v4();
    let jwt_secret = "test_secret_key";
    
    let token = auth_provider.generate_token(user_id).expect("Failed to generate token");
    let claims = verify_token(&token, jwt_secret).expect("Failed to verify token");
    
    // User ID in claims should match original
    assert_eq!(claims.sub, user_id.to_string());
}

#[test]
fn test_jwt_token_verification_failure() {
    let auth_provider = Argon2AuthProvider::new("test_secret_key".to_string());
    let user_id = Uuid::new_v4();
    let wrong_secret = "wrong_secret";
    
    let token = auth_provider.generate_token(user_id).expect("Failed to generate token");
    
    // Verification with wrong secret should fail
    assert!(verify_token(&token, wrong_secret).is_err());
}

#[test]
fn test_jwt_token_expiration() {
    let auth_provider = Argon2AuthProvider::new("test_secret_key".to_string());
    let user_id = Uuid::new_v4();
    let jwt_secret = "test_secret_key";
    
    // Generate token
    let token = auth_provider.generate_token(user_id).expect("Failed to generate token");
    
    // Token should be valid immediately
    assert!(verify_token(&token, jwt_secret).is_ok());
    
    // Note: We can't easily test actual expiration in unit tests without mocking time
    // This would be better tested in integration tests
}
